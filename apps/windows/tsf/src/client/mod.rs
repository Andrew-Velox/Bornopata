//! The DLL's engine layer: no engine here, only a connection to the Server that turns key presses and focus changes
//! into protocol messages. [`EngineClient`] works on any byte stream, so it is tested without Windows; `pipe` opens the
//! named pipe on Windows.

mod decision;
mod engine;
#[cfg(windows)]
pub mod pipe;
mod reply;

pub use decision::Decision;
pub use engine::EngineClient;
pub use reply::Reply;
