//! The layer-shell candidate window on Wayland, as a sink for the Router's frames.

pub use borno_server::panel::{PanelSink, render_frame};

use borno_panel::compositor::CursorRect as WindowCursor;
use borno_panel::{Panel, PanelConfig as WindowConfig, Rect};
use borno_platform::protocol::CursorRect;
use borno_platform::{PanelConfig, PanelLayout, PanelMode};

pub fn render_rect(cursor: CursorRect) -> WindowCursor {
    WindowCursor {
        rect: Rect::new(cursor.x, cursor.y, cursor.width, cursor.height),
        surface_relative: cursor.surface_relative,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cursor_rectangle_keeps_its_frame_of_reference() {
        let rect = render_rect(CursorRect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
            surface_relative: true,
        });
        assert_eq!(rect.rect, Rect::new(1.0, 2.0, 3.0, 4.0));
        assert!(rect.surface_relative);
    }
}
