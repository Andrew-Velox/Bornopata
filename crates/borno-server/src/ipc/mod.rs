//! What every Server's connection layer shares: one shell connection as a message loop ([`serve_stream`]), and the
//! loop that hands the connections' messages to the one [`Router`] ([`run_router`]). The listener, the peer check and
//! the popup channel are the platform's business.

mod connection;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::time::Instant;

use borno_platform::protocol::{ClientMessage, ServerMessage};

use crate::router::Router;

pub use connection::serve_stream;

/// A message for the Router with the channel for its answer.
pub type Request = (ClientMessage, mpsc::Sender<Option<ServerMessage>>);

/// Connections beyond this many are dropped; a broken client cannot exhaust threads.
pub const MAX_CONNECTIONS: usize = 64;

/// Requests waiting for the Router; bounded so a flooding client cannot grow memory.
pub const QUEUE: usize = 128;

/// Why [`run_router`] returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exit {
    /// The stop flag was set.
    Stopped,

    /// The config changed and nobody is mid-word: the Server should start over with it.
    Reload,

    /// Every connection sender is gone.
    Disconnected,
}

/// Sends one message to the Router and waits for its answer.
pub fn ask(sender: &SyncSender<Request>, message: ClientMessage) -> Option<ServerMessage> {
    let (reply, answer) = mpsc::channel();
    sender.send((message, reply)).ok()?;
    answer.recv().ok().flatten()
}

/// Runs `router` on this thread: answers the requests of the connections and ticks it, until `stop` is set, the config
/// asks for a restart, or there are no more connections to wait for.
pub fn run_router(router: &mut Router, receiver: &Receiver<Request>, stop: &AtomicBool) -> Exit {
    // The tick deadline is absolute, so a steady stream of messages cannot push it back forever.
    let mut due = Instant::now() + router.next_tick();
    while !stop.load(Ordering::Relaxed) {
        let now = Instant::now();
        if now >= due {
            router.tick();
            if router.reload_due() {
                return Exit::Reload;
            }
            due = Instant::now() + router.next_tick();
            continue;
        }
        match receiver.recv_timeout(due - now) {
            Ok((message, reply)) => {
                let _ = reply.send(router.handle(message));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return Exit::Disconnected,
        }
    }
    Exit::Stopped
}
