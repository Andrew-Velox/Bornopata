//! The input logic every Server shares, with no sockets, no window system and no operating system API but the clock.
//!
//! [`Router`] is the whole input logic. A platform Server owns the connections and the windows and hands frames to
//! them through [`panel::PanelSink`] and [`popup::PopupSink`].

pub mod error;
pub mod panel;
pub mod popup;
pub mod router;
pub mod session;
pub mod shared;

pub use error::ServerError;
pub use router::Router;
pub use shared::Shared;
