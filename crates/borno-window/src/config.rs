//! How the window is configured when it starts.

use borno_render::Layout;

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
