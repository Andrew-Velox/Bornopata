//! The wire protocol between the Linux Server and a shell (the Fcitx5 plugin).
//!
//! Frames on the socket are a 4-byte little-endian length and a JSON body ([`write_message`], [`read_message`]).
//! The shell sends [`ClientMessage`]s; the Server answers each event with a [`ServerMessage`].
//! The Server owns all input state, the shell only forwards events and draws the [`Frame`] it gets back.

mod capabilities;
mod codec;
pub mod frame;
mod identity;
pub mod key;
mod message;
mod mode;
mod session;

pub use capabilities::Capabilities;
pub use codec::{CodecError, read_message, write_message};
pub use frame::{Frame, FrameCandidate, Source};
pub use identity::Identity;
pub use key::{KeyEvent, KeyModifiers, KeyOutcome};
pub use message::{ClientMessage, Event, ServerMessage};
pub use mode::Mode;
pub use session::SessionId;

/// Bumped whenever a message or a field changes meaning. The Server refuses a shell that sends another number;
/// there is no compatibility layer, the Server and the plugin are installed together.
pub const PROTOCOL_VERSION: u32 = 2;

#[cfg(test)]
mod tests;
