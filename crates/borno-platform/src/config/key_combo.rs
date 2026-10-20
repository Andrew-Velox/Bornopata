//! A shortcut: modifiers plus one key, written in the config as `ctrl+alt+space`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::Modifiers;
use crate::protocol::KeyEvent;

/// X11 keysym of the space bar, which is what the shells send as the virtual key.
const SPACE: u32 = 0x20;

/// The shells' virtual key for Delete. A printable `.` has the same code but carries a character.
const DELETE: u32 = 0x2e;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComboKey {
    Space,

    Delete,

    /// A lower-case ASCII letter or a digit.
    Char(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct KeyCombo {
    /// At least one modifier: a bare key would break typing.
    pub modifiers: Modifiers,

    pub key: ComboKey,
}

impl KeyCombo {
    /// Whether `event` is exactly this shortcut. Caps Lock is not a modifier here.
    pub fn matches(&self, event: &KeyEvent) -> bool {
        let held = &event.modifiers;
        if held.ctrl != self.modifiers.ctrl
            || held.alt != self.modifiers.alt
            || held.shift != self.modifiers.shift
            || held.win != self.modifiers.super_key
        {
            return false;
        }
        match self.key {
            ComboKey::Space => event.virtual_key == SPACE,
            ComboKey::Delete => event.virtual_key == DELETE && event.character.is_none(),
            ComboKey::Char(expected) => event
                .character
                .is_some_and(|typed| typed.eq_ignore_ascii_case(&expected)),
        }
    }

    /// Config spelling, such as `ctrl+alt+space`.
    pub fn spelling(&self) -> String {
        let key = match self.key {
            ComboKey::Space => "space".to_owned(),
            ComboKey::Delete => "delete".to_owned(),
            ComboKey::Char(c) => c.to_string(),
        };
        format!("{}+{key}", self.modifiers.spelling())
    }
}

impl FromStr for KeyCombo {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (modifiers, key) = text
            .trim()
            .rsplit_once('+')
            .ok_or_else(|| format!("expected modifiers+key, got {text:?}"))?;
        let modifiers: Modifiers = modifiers.parse()?;
        if modifiers.is_empty() {
            return Err(format!("a shortcut needs a modifier: {text:?}"));
        }
        let key = key.trim().to_ascii_lowercase();
        let key = if key == "space" {
            ComboKey::Space
        } else if key == "delete" {
            ComboKey::Delete
        } else {
            let mut chars = key.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) if c.is_ascii_alphanumeric() => ComboKey::Char(c),
                _ => {
                    return Err(format!(
                        "key must be space, delete, a letter or a digit: {key:?}"
                    ));
                }
            }
        };
        Ok(Self { modifiers, key })
    }
}

impl TryFrom<String> for KeyCombo {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        text.parse()
    }
}

impl From<KeyCombo> for String {
    fn from(combo: KeyCombo) -> Self {
        combo.spelling()
    }
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.spelling())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::KeyModifiers;

    fn event(key: u32, character: Option<char>, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(key, character, modifiers)
    }

    #[test]
    fn parses_and_prints_the_default() {
        let combo: KeyCombo = "ctrl+alt+space".parse().unwrap();
        assert_eq!(combo.key, ComboKey::Space);
        assert_eq!(combo.spelling(), "ctrl+alt+space");
        assert_eq!("alt+ctrl+SPACE".parse::<KeyCombo>().unwrap(), combo);
    }

    #[test]
    fn rejects_bare_keys_and_bad_keys() {
        assert!("space".parse::<KeyCombo>().is_err());
        assert!("ctrl+".parse::<KeyCombo>().is_err());
        assert!("ctrl+ab".parse::<KeyCombo>().is_err());
        assert!("ctrl+-".parse::<KeyCombo>().is_err());
        assert!("hyper+a".parse::<KeyCombo>().is_err());
        assert!("+a".parse::<KeyCombo>().is_err());
    }

    #[test]
    fn matches_exactly_the_modifiers_and_key() {
        let combo: KeyCombo = "ctrl+alt+space".parse().unwrap();
        let chord = KeyModifiers {
            ctrl: true,
            alt: true,
            ..KeyModifiers::default()
        };
        assert!(combo.matches(&event(0x20, Some(' '), chord)));
        let with_caps = KeyModifiers {
            caps: true,
            ..chord
        };
        assert!(combo.matches(&event(0x20, Some(' '), with_caps)));
        let extra = KeyModifiers {
            shift: true,
            ..chord
        };
        assert!(!combo.matches(&event(0x20, Some(' '), extra)));
        assert!(!combo.matches(&event(0x20, Some(' '), KeyModifiers::default())));
        assert!(!combo.matches(&event(0x61, Some('a'), chord)));
    }

    #[test]
    fn delete_is_a_key_without_a_character() {
        let combo: KeyCombo = "shift+delete".parse().unwrap();
        assert_eq!(combo.spelling(), "shift+delete");
        let shift = KeyModifiers {
            shift: true,
            ..KeyModifiers::default()
        };
        assert!(combo.matches(&event(0x2e, None, shift)));
        assert!(!combo.matches(&event(0x2e, Some('.'), shift)));
        assert!(!combo.matches(&event(0x2e, None, KeyModifiers::default())));
    }

    #[test]
    fn letter_shortcuts_ignore_the_case_of_the_typed_character() {
        let combo: KeyCombo = "ctrl+shift+b".parse().unwrap();
        let chord = KeyModifiers {
            ctrl: true,
            shift: true,
            ..KeyModifiers::default()
        };
        assert!(combo.matches(&event(0x42, Some('B'), chord)));
        assert!(combo.matches(&event(0x62, Some('b'), chord)));
        assert!(!combo.matches(&event(0x63, Some('c'), chord)));
    }
}
