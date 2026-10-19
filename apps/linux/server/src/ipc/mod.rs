//! The Unix socket: accepts shells of the same user and moves messages between them and the [`Router`].
//!
//! Connections each get a thread that only reads, validates and forwards; the Router runs on the thread that
//! called [`serve`], so input logic never needs a lock.

mod connection;
mod socket;

use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, SyncSender};
use std::thread;
use std::time::Instant;

use borno_platform::protocol::{ClientMessage, ServerMessage};

use crate::popup::PopupHub;
use crate::router::Router;

pub use socket::{bind_socket, socket_path};

/// A message for the Router with the channel for its answer.
type Request = (ClientMessage, mpsc::Sender<Option<ServerMessage>>);

/// Connections beyond this many are dropped; a broken client cannot exhaust threads.
const MAX_CONNECTIONS: usize = 64;

/// Requests waiting for the Router; bounded so a flooding client cannot grow memory.
const QUEUE: usize = 128;

static STOP: AtomicBool = AtomicBool::new(false);
static CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

/// Asks [`serve`] to return at its next tick; safe to call from a signal handler.
pub fn request_shutdown() {
    STOP.store(true, Ordering::Relaxed);
}

/// Serves `router` on `path` until [`request_shutdown`], without popup channels. Removes the socket file on the way out.
pub fn serve(path: impl AsRef<Path>, router: &mut Router) -> io::Result<()> {
    serve_with_popup(path, router, None)
}

/// Like [`serve`], and shells may attach popup channels when `popup` is the hub that draws for them.
pub fn serve_with_popup(
    path: impl AsRef<Path>,
    router: &mut Router,
    popup: Option<PopupHub>,
) -> io::Result<()> {
    let listener = bind_socket(path.as_ref())?;
    let (sender, receiver) = mpsc::sync_channel::<Request>(QUEUE);
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if CONNECTIONS.fetch_add(1, Ordering::Relaxed) >= MAX_CONNECTIONS {
                        CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
                        continue;
                    }
                    let sender = sender.clone();
                    let popup = popup.clone();
                    thread::spawn(move || {
                        connection::serve_connection(stream, &sender, popup.as_ref());
                        CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
                    });
                }
                Err(error) => {
                    tracing::error!(%error, "accepting a connection failed");
                    break;
                }
            }
        }
    });
    // The tick deadline is absolute, so a steady stream of messages cannot push it back forever.
    let mut due = Instant::now() + router.next_tick();
    while !STOP.load(Ordering::Relaxed) {
        let now = Instant::now();
        if now >= due {
            router.tick();
            due = Instant::now() + router.next_tick();
            continue;
        }
        match receiver.recv_timeout(due - now) {
            Ok((message, reply)) => {
                let _ = reply.send(router.handle(message));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    std::fs::remove_file(path.as_ref())?;
    Ok(())
}

/// Sends one message to the Router and waits for its answer.
fn ask(sender: &SyncSender<Request>, message: ClientMessage) -> Option<ServerMessage> {
    let (reply, answer) = mpsc::channel();
    sender.send((message, reply)).ok()?;
    answer.recv().ok().flatten()
}
