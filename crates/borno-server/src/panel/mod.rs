//! The self-drawn candidate window as the Router sees it: a sink for frames, and the conversion from the
//! protocol's frame to what the renderer draws.

mod convert;

pub use convert::render_frame;

use borno_platform::protocol::CursorRect;

/// Where the Router sends the frame it wants drawn. A fake implements it in tests.
pub trait PanelSink: Send {
    /// Shows `frame` next to `cursor`. Returns `false` when the window is gone for good (the frame was not shown
    /// and the Router should stop using it).
    fn show(&mut self, frame: borno_render::Frame, cursor: CursorRect) -> bool;

    fn hide(&mut self);

    /// Whether the user has pinned the window: it then shows at the pinned place whatever the text cursor does, and
    /// needs no cursor position at all.
    fn pinned(&self) -> bool {
        false
    }

    /// The pin shortcut: pins the window at a place the user can drag it from, or frees it to follow the text cursor.
    fn toggle_pin(&mut self) {}
}
