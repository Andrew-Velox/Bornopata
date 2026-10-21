//! One shell connection on the Unix socket: the peer must be the same user, the shared loop does the rest.

use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::sync::mpsc::SyncSender;
use std::time::Duration;

use borno_platform::protocol::{
    PROTOCOL_VERSION, PopupClientMessage, ServerMessage, read_message, write_message,
};
use borno_server::ipc::{Request, serve_stream};

use crate::popup::PopupHub;

pub(super) fn serve_connection(
    stream: UnixStream,
    sender: &SyncSender<Request>,
    popup: Option<&PopupHub>,
) {
    if !peer_is_owner(&stream) {
        return;
    }
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
    if let Some((stream, token)) = serve_stream(stream, sender)
        && let Some(hub) = popup
    {
        serve_popup_channel(stream, hub, token);
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
