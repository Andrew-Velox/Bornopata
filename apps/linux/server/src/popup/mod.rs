//! Candidate frames drawn into the input-method popup surface of the shell.
//!
//! The Router hands frames to a [`PopupSink`]; the real one, [`PopupHub`], draws them on its own thread and pushes the
//! bitmaps over the popup channel, so typing never waits for a render. Design: `docs/design/popup-surface.md`.

mod error;
mod hub;
mod pixels;
mod pool;
mod wire;
mod worker;

pub use error::PopupError;
pub use hub::PopupHub;

use borno_platform::{PanelConfig, PanelMode};

/// Starts the render thread for the shells' popup surfaces, unless the config keeps them off or it cannot start
/// (no usable font or theme), which is logged and leaves the layer-shell window and Fcitx5's panel.
pub fn start(config: &PanelConfig) -> Option<PopupHub> {
    if matches!(config.mode, PanelMode::Fcitx | PanelMode::Native) {
        return None;
    }
    match PopupHub::start(config) {
        Ok(hub) => {
            tracing::info!(theme = %config.theme, "candidate popup: ready for shells that can show it");
            Some(hub)
        }
        Err(error) => {
            tracing::warn!(%error, "the candidate popup could not start");
            None
        }
    }
}

/// Where the Router sends the frame the shell should show in its popup. A fake implements it in tests.
pub trait PopupSink: Send {
    /// Shows `frame` in the popup of channel `token` for input context `context`. `false` when the channel is gone:
    /// the frame was not shown and the shell has to draw it.
    fn show(&mut self, token: u64, context: &str, frame: borno_render::Frame) -> bool;

    fn hide(&mut self, token: u64);

    /// Whether a shell has a popup channel attached with `token`.
    fn attached(&self, token: u64) -> bool;
}
