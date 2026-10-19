//! One shell connection: validates what it sends and forwards it to the Router under connection-unique session numbers.
//!
//! Anything unexpected closes the connection; the shell reconnects with a fresh handshake on its next key.

use std::collections::HashMap;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::SyncSender;
use std::time::Duration;

use borno_platform::protocol::{
    ClientMessage, Event, PROTOCOL_VERSION, PopupClientMessage, ServerMessage, SessionId,
    read_message, write_message,
};

use super::{Request, ask};
use crate::popup::PopupHub;

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

pub(super) fn serve_connection(
    mut stream: UnixStream,
    sender: &SyncSender<Request>,
    popup: Option<&PopupHub>,
) {
    if !peer_is_owner(&stream) {
        return;
    }
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
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
                if let Some(hub) = popup {
                    serve_popup_channel(stream, hub, token);
                }
                return;
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
}

/// A connection that attached as a popup channel: the render thread writes to it, this thread reads what the shell
/// reports until the shell goes away.
fn serve_popup_channel(mut stream: UnixStream, hub: &PopupHub, token: u64) {
    let Ok(writer) = stream.try_clone() else {
        return;
    };
    // The shell must have the reply before any bitmap, so the render thread gets the stream only after it.
    if write_message(
        &mut stream,
        &ServerMessage::Attached {
            protocol: PROTOCOL_VERSION,
        },
    )
    .is_err()
    {
        return;
    }
    hub.attach(token, writer);
    while let Ok(Some(message)) = read_message::<_, PopupClientMessage>(&mut stream) {
        match message {
            PopupClientMessage::Released { buffer } => hub.released(token, buffer),
            PopupClientMessage::Scale { scale_120 } => hub.scale(token, scale_120),
        }
    }
    hub.detach(token);
}

/// Whether the other end of the socket is the same user.
fn peer_is_owner(stream: &UnixStream) -> bool {
    #[cfg(target_os = "linux")]
    {
        let mut credentials = libc::ucred {
            pid: 0,
            uid: 0,
            gid: 0,
        };
        let mut size = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
        unsafe {
            libc::getsockopt(
                stream.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                (&mut credentials as *mut libc::ucred).cast(),
                &mut size,
            ) == 0
                && credentials.uid == libc::geteuid()
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let mut uid: libc::uid_t = 0;
        let mut gid: libc::gid_t = 0;
        unsafe {
            libc::getpeereid(stream.as_raw_fd(), &mut uid, &mut gid) == 0 && uid == libc::geteuid()
        }
    }
}
