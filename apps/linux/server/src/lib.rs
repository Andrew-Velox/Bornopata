//! Linux Server: holds the Bangla engines and serves the Fcitx5 plugin over a Unix socket.
//!
//! [`Router`] is the whole input logic and knows nothing about sockets; [`ipc`] only moves messages.

pub mod error;
#[cfg(unix)]
pub mod ipc;
pub mod router;
pub mod session;
pub mod shared;

pub use error::ServerError;
pub use router::Router;
pub use shared::Shared;
