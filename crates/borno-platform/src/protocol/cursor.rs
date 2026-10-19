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
