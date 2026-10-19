//! The self-drawn candidate window as the Router sees it: a sink for frames, and the conversion from the
//! protocol's frame to what the renderer draws.

mod convert;

pub use convert::{render_frame, render_rect};

use borno_panel::{Panel, PanelConfig as WindowConfig};
use borno_platform::protocol::CursorRect;
use borno_platform::{PanelConfig, PanelLayout, PanelMode};

/// Where the Router sends the frame it wants drawn. A fake implements it in tests.
pub trait PanelSink: Send {
    /// Shows `frame` next to `cursor`. Returns `false` when the window is gone for good (the frame was not shown
    /// and the Router should stop using it).
    fn show(&mut self, frame: borno_render::Frame, cursor: CursorRect) -> bool;

    fn hide(&mut self);
}

struct NativePanel(Panel);

impl PanelSink for NativePanel {
    fn show(&mut self, frame: borno_render::Frame, cursor: CursorRect) -> bool {
        self.0.show(frame, render_rect(cursor)).is_ok() && self.0.is_alive()
    }

    fn hide(&mut self) {
        let _ = self.0.hide();
    }
}

/// Starts the window the config asks for. `None` means Fcitx5's own panel does the drawing: the config says so, or
/// the compositor cannot show the window (no Wayland session, no layer-shell), which is logged.
pub fn start(config: &PanelConfig) -> Option<Box<dyn PanelSink>> {
    match config.mode {
        PanelMode::Fcitx => {
            tracing::info!("candidate window: Fcitx5 panel (configured)");
            return None;
        }
        PanelMode::Popup => {
            tracing::info!("candidate window: input-method popup only (configured)");
            return None;
        }
        PanelMode::Auto | PanelMode::Native => {}
    }
    let window = WindowConfig {
        theme: config.theme.clone(),
        dark: config.dark,
        layout: match config.layout {
            PanelLayout::Vertical => borno_render::Layout::Vertical,
            PanelLayout::Horizontal => borno_render::Layout::Horizontal,
        },
    };
    match Panel::start(window) {
        Ok(panel) => {
            tracing::info!(theme = %config.theme, "candidate window: self-drawn");
            Some(Box::new(NativePanel(panel)))
        }
        Err(error) if config.mode == PanelMode::Native => {
            tracing::warn!(%error, "the self-drawn candidate window could not start, using the Fcitx5 panel");
            None
        }
        Err(error) => {
            tracing::info!(%error, "candidate window: Fcitx5 panel (the self-drawn one is not available here)");
            None
        }
    }
}
