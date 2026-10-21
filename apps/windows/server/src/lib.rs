//! Windows Server: holds the Bangla engines, serves the text service DLL over a named pipe and draws the candidate
//! window.
//!
//! The input logic is `borno-server`, shared with the other platforms. The pipe ([`pipe`]) and the window ([`window`])
//! need Windows, so the crate is empty elsewhere.

#[cfg(windows)]
pub mod pipe;
#[cfg(windows)]
pub mod window;
