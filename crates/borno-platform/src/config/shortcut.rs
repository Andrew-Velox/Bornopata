//! The `[shortcut]` section.

use serde::{Deserialize, Serialize};

use super::KeyCombo;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutConfig {
    /// Switches between Bangla and English typing.
    pub toggle: KeyCombo,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            toggle: "ctrl+alt+space"
                .parse()
                .expect("the default shortcut is valid"),
        }
    }
}
