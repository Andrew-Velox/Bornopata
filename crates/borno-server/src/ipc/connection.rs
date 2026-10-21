//! One shell connection: validates what it sends and forwards it to the Router under connection-unique session numbers.
//!
//! Anything unexpected closes the connection; the shell reconnects with a fresh handshake on its next key.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::SyncSender;

use borno_platform::protocol::{
    ClientMessage, Event, PROTOCOL_VERSION, ServerMessage, SessionId, read_message, write_message,
};

use super::{Request, ask};

/// Longest context identifier accepted.
const MAX_CONTEXT: usize = 64;

/// Session numbers are global to the Server: two shells may both use number 1.
static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);

struct Mapped {
    global: SessionId,

    /// The first event of a session must be its capabilities, so a key is never handled before the Server knows
    /// whether the field is private.
    capabilities_seen: bool,
}

/// Reads messages from `stream` and answers them until the shell goes away or breaks the rules; its sessions are then
/// closed. Returns the stream and the token when the shell asked to attach a popup channel instead: the caller decides
/// what to do with it (the sessions have been closed by then, there are none on an attaching connection).
pub fn serve_stream<S: Read + Write>(
    mut stream: S,
    sender: &SyncSender<Request>,
) -> Option<(S, u64)> {
    let mut sessions = HashMap::<SessionId, Mapped>::new();
    while let Ok(Some(message)) = read_message::<_, ClientMessage>(&mut stream) {
        let reply = match message {
            ClientMessage::Open {
                session: local,
                app,
                protocol,
                generation,
                context,
                popup: token,
            } => {
                if protocol != PROTOCOL_VERSION
                    || generation == 0
                    || context.is_empty()
                    || context.len() > MAX_CONTEXT
                    || sessions.contains_key(&local)
                {
                    break;
                }
                let global = SessionId(NEXT_SESSION.fetch_add(1, Ordering::Relaxed));
                sessions.insert(
                    local,
                    Mapped {
                        global,
                        capabilities_seen: false,
                    },
                );
                ask(
                    sender,
                    ClientMessage::Open {
                        session: global,
                        app,
                        protocol,
                        generation,
                        context,
                        popup: token,
                    },
                )
                .map(|reply| reply.with_session(local))
            }
            ClientMessage::Attach { protocol, token } => {
                if protocol != PROTOCOL_VERSION || !sessions.is_empty() {
                    break;
                }
                return Some((stream, token));
            }
            ClientMessage::Close { session: local } => {
                if let Some(mapped) = sessions.remove(&local) {
                    ask(
                        sender,
                        ClientMessage::Close {
                            session: mapped.global,
                        },
                    );
                }
                None
            }
            ClientMessage::Event {
                session: local,
                event,
            } => {
                let Some(mapped) = sessions.get_mut(&local) else {
                    // A late callback for a session that is already closed still has a caller waiting.
                    let ignored = ServerMessage::Ignored { session: local };
                    if write_message(&mut stream, &ignored).is_err() {
                        break;
                    }
                    continue;
                };
                let is_capabilities = matches!(event, Event::Capabilities(_));
                if !mapped.capabilities_seen && !is_capabilities {
                    break;
                }
                mapped.capabilities_seen = true;
                ask(
                    sender,
                    ClientMessage::Event {
                        session: mapped.global,
                        event,
                    },
                )
                .map(|reply| reply.with_session(local))
            }
        };
        if let Some(reply) = reply
            && write_message(&mut stream, &reply).is_err()
        {
            break;
        }
    }
    for (_, mapped) in sessions {
        ask(
            sender,
            ClientMessage::Close {
                session: mapped.global,
            },
        );
    }
    None
}
