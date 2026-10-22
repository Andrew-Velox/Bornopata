//! The `[shortcut]` section.

use serde::{Deserialize, Serialize};

use super::KeyCombo;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutConfig {
    /// Switches between Bangla and English typing.
    pub toggle: KeyCombo,

    /// While a word is being typed, takes back what was learned about the highlighted candidate; on a next-word
    /// suggestion, stops it being offered. Needs a modifier like every shortcut.
    pub forget: KeyCombo,

    /// While the candidate window shows, pins it where the user can drag it, or lets it follow the text cursor again.
    pub pin: KeyCombo,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            toggle: "ctrl+alt+space"
                .parse()
                .expect("the default shortcut is valid"),
            forget: "shift+delete"
                .parse()
                .expect("the default shortcut is valid"),
            pin: "ctrl+alt+m".parse().expect("the default shortcut is valid"),
        }
    }
}
