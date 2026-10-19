//! Sending a popup channel message, with a file descriptor when it has one.

use std::io::{self, IoSlice, Write};
use std::os::fd::BorrowedFd;
use std::os::unix::net::UnixStream;

use borno_platform::protocol::{CodecError, write_message};
use rustix::net::{SendAncillaryBuffer, SendAncillaryMessage, SendFlags, sendmsg};
use serde::Serialize;

/// Writes one framed message. `fd` goes with the first byte as `SCM_RIGHTS`, so the shell has it when it reads the
/// message.
pub fn send_message<T: Serialize>(
    stream: &mut UnixStream,
    message: &T,
    fd: Option<BorrowedFd<'_>>,
) -> Result<(), CodecError> {
    let Some(fd) = fd else {
        return write_message(stream, message);
    };
    let mut framed = Vec::new();
    write_message(&mut framed, message)?;
    let fds = [fd];
    let mut space = [std::mem::MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
    let mut control = SendAncillaryBuffer::new(&mut space);
    control.push(SendAncillaryMessage::ScmRights(&fds));
    let sent = sendmsg(
        &*stream,
        &[IoSlice::new(&framed)],
        &mut control,
        SendFlags::NOSIGNAL,
    )
    .map_err(io::Error::from)?;
    stream.write_all(&framed[sent..])?;
    stream.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{IoSliceMut, Read, Seek, SeekFrom};
    use std::os::fd::AsFd;

    use borno_platform::protocol::{PopupServerMessage, read_message};
    use rustix::net::{RecvAncillaryBuffer, RecvAncillaryMessage, RecvFlags, recvmsg};

    use super::*;

    /// Reads the first bytes of a stream together with the descriptors that came with them.
    fn receive_with_fd(stream: &UnixStream, length: usize) -> (Vec<u8>, Option<File>) {
        let mut bytes = vec![0u8; length];
        let mut space = [std::mem::MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
        let mut control = RecvAncillaryBuffer::new(&mut space);
        let received = recvmsg(
            stream,
            &mut [IoSliceMut::new(&mut bytes)],
            &mut control,
            RecvFlags::CMSG_CLOEXEC,
        )
        .unwrap();
        bytes.truncate(received.bytes);
        let fd = control.drain().find_map(|message| match message {
            RecvAncillaryMessage::ScmRights(mut fds) => fds.next(),
            _ => None,
        });
        (bytes, fd.map(File::from))
    }

    #[test]
    fn a_message_without_a_descriptor_is_an_ordinary_frame() {
        let (mut server, mut shell) = UnixStream::pair().unwrap();
        send_message(&mut server, &PopupServerMessage::Hide, None).unwrap();
        let read: PopupServerMessage = read_message(&mut shell).unwrap().unwrap();
        assert_eq!(read, PopupServerMessage::Hide);
    }

    #[test]
    fn the_descriptor_arrives_with_the_message_and_is_the_same_file() {
        let (mut server, shell) = UnixStream::pair().unwrap();
        let mut backing = tempfile_with(b"slot");
        let message = PopupServerMessage::Pool { pool: 3, size: 4 };
        send_message(&mut server, &message, Some(backing.as_fd())).unwrap();
        let (bytes, fd) = receive_with_fd(&shell, 64);
        assert!(bytes.len() > 4);
        let read: PopupServerMessage = read_message(&mut &bytes[..]).unwrap().unwrap();
        assert_eq!(read, message);
        let mut received = fd.expect("the descriptor came with the first bytes");
        let mut text = String::new();
        received.seek(SeekFrom::Start(0)).unwrap();
        received.read_to_string(&mut text).unwrap();
        assert_eq!(text, "slot");
        backing.write_all(b"!").unwrap();
    }

    fn tempfile_with(content: &[u8]) -> File {
        let fd = rustix::fs::memfd_create("test", rustix::fs::MemfdFlags::CLOEXEC).unwrap();
        let mut file = File::from(fd);
        file.write_all(content).unwrap();
        file
    }
}
