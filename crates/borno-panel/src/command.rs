//! What the Server asks the panel to do.

use borno_render::{Frame, Layout};

use crate::compositor::CursorRect;

/// How the panel is configured when it starts.
#[derive(Debug, Clone, PartialEq)]
pub struct PanelConfig {
    /// Built-in theme id (`bornopata`, `sakura`) or the directory of a user theme.
    pub theme: String,

    /// Use the dark variant of themes that have one.
    pub dark: bool,

    pub layout: Layout,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            theme: "bornopata".to_owned(),
            dark: false,
            layout: Layout::Vertical,
        }
    }
}

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
