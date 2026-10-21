//! Asking the compositor where things are.
//!
//! A Wayland client only knows its text cursor relative to its own surface. To put a popup next to it the popup
//! needs the global position of that window, and the compositor's own IPC is the one source for it.

mod hyprland;

pub use hyprland::Hyprland;

use borno_window::Rect;

/// A text cursor rectangle as the input method framework reports it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorRect {
    pub rect: Rect,

    /// The rectangle is relative to the focused window's surface (Wayland clients). When `false` it is already
    /// in global coordinates (X11 clients through XWayland).
    pub surface_relative: bool,
}

/// Where the cursor ended up on the screen.
#[derive(Debug, Clone, PartialEq)]
pub struct Located {
    /// The text cursor rectangle in global logical coordinates.
    pub cursor: Rect,

    /// The monitor it is on, in global logical coordinates.
    pub monitor: Rect,

    /// The monitor's name as `wl_output` reports it, to put the popup on the same output.
    pub monitor_name: String,

    /// The monitor's scale factor.
    pub scale: f32,
}

pub trait Compositor: Send {
    /// Resolves `cursor` to global coordinates and the monitor it is on; `None` when the compositor cannot say.
    fn locate(&mut self, cursor: CursorRect) -> Option<Located>;
}

/// The compositor of the running session, if it is one this crate knows.
pub fn detect() -> Option<Box<dyn Compositor>> {
    Hyprland::from_env().map(|hyprland| Box::new(hyprland) as Box<dyn Compositor>)
}
