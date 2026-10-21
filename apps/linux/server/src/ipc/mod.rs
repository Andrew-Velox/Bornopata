//! The Unix socket: accepts shells of the same user and moves messages between them and the [`Router`].
//!
//! Connections each get a thread that only reads, validates and forwards; the Router runs on the thread that
//! called [`serve`], so input logic never needs a lock.

mod connection;
mod socket;

use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;

use borno_server::ipc::{Exit, MAX_CONNECTIONS, QUEUE, Request, run_router};

use crate::popup::PopupHub;
use crate::router::Router;

pub use socket::{bind_socket, socket_path};

static STOP: AtomicBool = AtomicBool::new(false);
static RELOAD: AtomicBool = AtomicBool::new(false);
static CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

/// Asks [`serve`] to return at its next tick; safe to call from a signal handler.
pub fn request_shutdown() {
    STOP.store(true, Ordering::Relaxed);
}

/// Whether [`serve`] returned because the config changed and the Server should start over (not because it was asked
/// to stop).
pub fn reload_requested() -> bool {
    RELOAD.load(Ordering::Relaxed)
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
    if run_router(router, &receiver, &STOP) == Exit::Reload {
        RELOAD.store(true, Ordering::Relaxed);
    }
    std::fs::remove_file(path.as_ref())?;
    Ok(())
}
