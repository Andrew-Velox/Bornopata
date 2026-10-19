//! Where the text cursor is.

use serde::{Deserialize, Serialize};

/// The text cursor rectangle as the framework reports it, in logical pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct CursorRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,

    /// The coordinates are relative to the focused window's surface (Wayland). `false` means global screen
    /// coordinates (X11).
    pub surface_relative: bool,
}

impl CursorRect {
    /// `false` for the all-zero rectangle some Wayland compositors hand over when the client never reported a caret
    /// position (Fcitx5 on Hyprland with Chromium): a window placed by it would sit in the corner of the app.
    pub fn is_known(&self) -> bool {
        self.x != 0.0 || self.y != 0.0 || self.width != 0.0 || self.height != 0.0
    }
}
