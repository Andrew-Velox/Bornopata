//! The shell's end of a popup channel: messages from the Server together with the descriptors that came with them.

use std::collections::VecDeque;
use std::fs::File;
use std::io::IoSliceMut;
use std::os::unix::net::UnixStream;

use borno_platform::protocol::{PopupFrame, PopupServerMessage};
use rustix::net::{RecvAncillaryBuffer, RecvAncillaryMessage, RecvFlags, recvmsg};

/// The shell's end of a popup channel.
pub struct Shell {
    pub stream: UnixStream,
    buffer: Vec<u8>,
    pub fds: VecDeque<File>,
}

impl Shell {
    pub fn new(stream: UnixStream) -> Self {
        Self {
            stream,
            buffer: Vec::new(),
            fds: VecDeque::new(),
        }
    }
}

impl Shell {
    pub fn next(&mut self) -> Option<PopupServerMessage> {
        loop {
            if self.buffer.len() >= 4 {
                let length = u32::from_le_bytes(self.buffer[..4].try_into().unwrap()) as usize;
                if self.buffer.len() >= 4 + length {
                    let body: Vec<u8> = self.buffer.drain(..4 + length).skip(4).collect();
                    return Some(serde_json::from_slice(&body).unwrap());
                }
            }
            let mut bytes = [0u8; 4096];
            let mut space = [std::mem::MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(4))];
            let mut control = RecvAncillaryBuffer::new(&mut space);
            let received = recvmsg(
                &self.stream,
                &mut [IoSliceMut::new(&mut bytes)],
                &mut control,
                RecvFlags::CMSG_CLOEXEC,
            )
            .ok()?;
            if received.bytes == 0 {
                return None;
            }
            self.buffer.extend_from_slice(&bytes[..received.bytes]);
            for message in control.drain() {
                if let RecvAncillaryMessage::ScmRights(fds) = message {
                    self.fds.extend(fds.map(File::from));
                }
            }
        }
    }

    pub fn present(&mut self) -> PopupFrame {
        loop {
            match self.next().expect("the Server sent something in time") {
                PopupServerMessage::Present(frame) => return frame,
                PopupServerMessage::Pool { .. } | PopupServerMessage::Hide => {}
            }
        }
    }
}
