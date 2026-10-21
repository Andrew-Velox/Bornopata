//! Windows only: opens the Server's named pipe as a byte stream for [`EngineClient`](super::EngineClient).

use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::fs::OpenOptionsExt;

use borno_platform::protocol::DEFAULT_PIPE_NAME;
use windows::Win32::Foundation::ERROR_PIPE_BUSY;
use windows::Win32::System::Pipes::WaitNamedPipeW;
use windows::core::HSTRING;

/// A connected pipe. The other end closing reads as the end of the stream; `flush` does nothing, since flushing a pipe
/// blocks until the other end has read everything.
pub type PipeStream = File;

/// How long to wait for a free instance when all are taken, and how often to try. Short: this runs on the UI thread of
/// the application, and the system takes the input method away from one that blocks.
const BUSY_WAIT_MS: u32 = 300;
const BUSY_RETRIES: u32 = 1;

pub fn connect_default() -> io::Result<PipeStream> {
    connect(DEFAULT_PIPE_NAME)
}

pub fn connect(name: &str) -> io::Result<PipeStream> {
    let mut attempts = 0;
    loop {
        match OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(name)
        {
            Err(error)
                if error.raw_os_error() == Some(ERROR_PIPE_BUSY.0 as i32)
                    && attempts < BUSY_RETRIES =>
            {
                attempts += 1;
                let _ = unsafe { WaitNamedPipeW(&HSTRING::from(name), BUSY_WAIT_MS) };
            }
            result => return result,
        }
    }
}
