//! Modifier state of a key event.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,

    /// The Super (Windows) key.
    pub win: bool,

    /// Caps Lock is on (a lock, not a held key).
    #[serde(default)]
    pub caps: bool,
}

impl KeyModifiers {
    /// Ctrl, Alt or Super held. Shift does not count: it only changes the character.
    pub fn has_command_key(self) -> bool {
        self.ctrl || self.alt || self.win
    }
}
