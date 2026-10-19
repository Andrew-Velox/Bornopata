//! The popup channel: a second connection on which the Server pushes rendered bitmaps and the shell reports what the
//! compositor did with them. Design: `docs/design/popup-surface.md`.

mod client;
mod frame;
mod server;

pub use client::PopupClientMessage;
pub use frame::PopupFrame;
pub use server::PopupServerMessage;
