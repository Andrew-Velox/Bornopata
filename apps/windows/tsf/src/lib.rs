//! The Windows text service of Bornopata.
//!
//! Windows loads this DLL into every application that takes text input, so the engine is not in it: it lives in the
//! Server process (`borno-windows-server`). The DLL turns the system's keys into protocol messages, writes the text and
//! the word being typed into the document, and tells the Server where the cursor is; the Server draws the candidates.
//!
//! - [`client`]: the connection to the Server, independent of Windows and tested against the real Router.
//! - `com` (Windows only): the COM entry points, the TSF interfaces, edit sessions and self-registration.

pub mod client;
#[cfg(windows)]
mod com;
pub mod error;

pub use error::ClientError;
