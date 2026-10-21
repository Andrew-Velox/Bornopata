//! What the Server asks the panel to do.

use borno_render::Frame;

use crate::compositor::CursorRect;

#[derive(Debug)]
pub enum Command {
    /// Show this frame next to the text cursor.
    Show {
        frame: Box<Frame>,
        cursor: CursorRect,
    },

    /// Hide the window.
    Hide,

    /// Stop the panel thread.
    Shutdown,
}
