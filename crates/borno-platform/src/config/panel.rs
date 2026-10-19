//! The `[panel]` section: which candidate window is drawn.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PanelMode {
    /// The self-drawn window where the compositor allows it (the input-method popup for Wayland-native clients, the
    /// layer-shell window for X11 and XWayland ones), Fcitx5's own panel otherwise.
    #[default]
    Auto,

    /// Only the input-method popup; Fcitx5's own panel wherever it is not available.
    Popup,

    /// Only the layer-shell window, and a failure to start it is logged as a warning, not as a normal fallback.
    Native,

    /// Always Fcitx5's own panel.
    Fcitx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PanelLayout {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelConfig {
    pub mode: PanelMode,

    /// A built-in theme id (`bornopata`, `sakura`) or the path of a theme directory.
    pub theme: String,

    /// The dark variant of themes that have one.
    pub dark: bool,

    pub layout: PanelLayout,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            mode: PanelMode::default(),
            theme: "bornopata".to_owned(),
            dark: false,
            layout: PanelLayout::default(),
        }
    }
}
