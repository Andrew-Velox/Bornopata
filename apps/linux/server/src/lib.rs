//! Linux Server: holds the Bangla engines and serves the Fcitx5 plugin over a Unix socket.
//!
//! The input logic ([`Router`] and the parts it uses) is `borno-server`, shared with the other platforms; this crate adds
//! the Unix socket ([`ipc`]), the Wayland candidate window ([`panel`]) and the popup surfaces ([`popup`]).

#[cfg(unix)]
pub mod ipc;
pub mod panel;
pub mod popup;

pub use borno_server::{Router, ServerError, Shared, error, router, session, shared};
