//! One key as the shell reports it.

use serde::{Deserialize, Serialize};

use super::KeyModifiers;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEvent {
    /// The key: the X11 keysym for printable keys, and the Windows virtual-key value for the function keys the
    /// Fcitx5 plugin maps (`0x08` Backspace, `0x09` Tab, `0x0d` Enter, `0x1b` Escape, `0x25`-`0x28` arrows ...).
    /// The numbers are historical; both sides share one table in the plugin's `key/mapping.cpp`.
    pub virtual_key: u32,

    /// The character the key produces with the current modifiers; `None` for function and modifier keys.
    pub character: Option<char>,

    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    pub fn new(virtual_key: u32, character: Option<char>, modifiers: KeyModifiers) -> Self {
        Self {
            virtual_key,
            character,
            modifiers,
        }
    }
}
