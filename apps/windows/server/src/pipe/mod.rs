//! The named pipe `\\.\pipe\bornopata`: serves the text service DLL of every application over byte streams.
//!
//! Each application process keeps its own connection, also while it is not in focus, so connections cannot be served
//! one after the other: a newly focused application that cannot connect would block its UI thread and the system
//! would switch the input method away. A background loop makes a new pipe instance for every client and gives it a
//! thread; the Router stays on the calling thread, and the connections hand it their messages over a channel.

mod instance;

use std::io;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use borno_server::Router;
use borno_server::ipc::{Exit, MAX_CONNECTIONS, QUEUE, run_router, serve_stream};

use instance::{create_instance, security_descriptor, wait_client};

static STOP: AtomicBool = AtomicBool::new(false);
static CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

/// How long a starting Server waits for the pipe name when another Server still holds it, such as the one that has just
/// handed over to this one after a config change.
const TAKE_OVER: Duration = Duration::from_secs(5);

/// Asks [`serve`] to return at its next tick.
pub fn request_shutdown() {
    STOP.store(true, Ordering::Relaxed);
}

/// Serves `router` on the pipe `name` until it is asked to stop or the config asks for a restart. Fails when another
/// Server keeps the pipe: two Servers would each hold a part of the state and draw their own window.
pub fn serve(name: &str, router: &mut Router) -> io::Result<Exit> {
    let first = first_instance(name)?;
    let (sender, receiver) = mpsc::sync_channel(QUEUE);
    let name = name.to_owned();
    thread::spawn(move || accept_loop(&name, first, &sender));
    tracing::info!("listening on the named pipe");
    Ok(run_router(router, &receiver, &STOP))
}

/// The first pipe instance, which also claims the name. Retries for a few seconds: the Server that hands over to this
/// one is still shutting down.
fn first_instance(name: &str) -> io::Result<std::fs::File> {
    let until = Instant::now() + TAKE_OVER;
    loop {
        match create_instance(name, security_descriptor(), true) {
            Ok(first) => return Ok(first),
            Err(error) if instance::is_taken(&error) && Instant::now() < until => {
                thread::sleep(Duration::from_millis(200));
            }
            Err(error) if instance::is_taken(&error) => {
                return Err(io::Error::other(
                    "another Bornopata Server is running (the named pipe is taken)",
                ));
            }
            Err(error) => return Err(error),
        }
    }
}

/// Waits for a client on the instance it has, serves it on a thread, then makes the next instance. Stops when no
/// instance can be made.
fn accept_loop(
    name: &str,
    first: std::fs::File,
    sender: &mpsc::SyncSender<borno_server::ipc::Request>,
) {
    let descriptor = security_descriptor();
    let mut instance = Ok(first);
    loop {
        let stream = match instance.and_then(wait_client) {
            Ok(stream) => stream,
            Err(error) => {
                tracing::error!(%error, "the named pipe stopped accepting connections");
                break;
            }
        };
        if CONNECTIONS.fetch_add(1, Ordering::Relaxed) >= MAX_CONNECTIONS {
            CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
        } else {
            let sender = sender.clone();
            thread::spawn(move || {
                serve_stream(stream, &sender);
                CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
            });
        }
        instance = create_instance(name, descriptor, false);
    }
}
