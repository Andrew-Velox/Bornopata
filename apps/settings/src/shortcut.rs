//! Recording a shortcut: which ones can be recorded, whether the pressed key is usable, and how it is written.

use crate::i18n::t;
use bamboo_slip::{KeyCode, KeyPress};
use borno_platform::{Config, KeyCombo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shortcut {
    /// Switches between Bangla and English.
    Toggle,

    /// Forgets the highlighted candidate or suggestion.
    Forget,
}

impl Shortcut {
    pub const ALL: [Self; 2] = [Self::Toggle, Self::Forget];

    pub fn title(self) -> &'static str {
        match self {
            Self::Toggle => t("Switch between Bangla and English"),
            Self::Forget => t("Forget a word"),
        }
    }

    pub fn info(self) -> &'static str {
        match self {
            Self::Toggle => t("Pressed anywhere you type. English mode lets every key through."),
            Self::Forget => t(
                "Press it on a highlighted word, or on a next-word suggestion, to take back what Bornopata learned about it.",
            ),
        }
    }

    /// Where it is in the config file.
    pub fn config_key(self) -> (&'static str, &'static str) {
        match self {
            Self::Toggle => ("shortcut", "toggle"),
            Self::Forget => ("shortcut", "forget"),
        }
    }

    pub fn default_value(self) -> &'static str {
        match self {
            Self::Toggle => "ctrl+alt+space",
            Self::Forget => "shift+delete",
        }
    }

    pub fn prompt(self) -> &'static str {
        t("Press the new keys")
    }

    /// The current value, written for people.
    pub fn current(self, config: &Config) -> String {
        label(&self.combo(config))
    }

    fn combo(self, config: &Config) -> KeyCombo {
        match self {
            Self::Toggle => config.shortcut.toggle,
            Self::Forget => config.shortcut.forget,
        }
    }

    /// The value to write for the keys pressed, or why they cannot be used (recording goes on waiting).
    pub fn accept(self, press: &KeyPress, config: &Config) -> Result<String, String> {
        let mut parts = Vec::new();
        if press.modifiers.control {
            parts.push("ctrl");
        }
        if press.modifiers.alt {
            parts.push("alt");
        }
        if press.modifiers.shift {
            parts.push("shift");
        }
        if press.modifiers.logo {
            parts.push("super");
        }
        if parts.is_empty() {
            return Err(t("Hold Ctrl, Alt, Shift or Super while you press the key").to_owned());
        }
        let key = match &press.key {
            KeyCode::Space => "space".to_owned(),
            KeyCode::Delete => "delete".to_owned(),
            KeyCode::Character(text) => {
                let mut chars = text.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) if c.is_ascii_alphanumeric() => {
                        c.to_ascii_lowercase().to_string()
                    }
                    _ => return Err(t("Use Space, Delete, a letter or a digit").to_owned()),
                }
            }
            _ => return Err(t("Use Space, Delete, a letter or a digit").to_owned()),
        };
        parts.push(&key);
        let spelling = parts.join("+");
        let combo: KeyCombo = spelling.parse().map_err(|error: String| error)?;
        let other = match self {
            Self::Toggle => Self::Forget,
            Self::Forget => Self::Toggle,
        };
        if other.combo(config) == combo {
            return Err(format!("{} “{}”", t("Already used for"), other.title()));
        }
        Ok(spelling)
    }
}

/// `ctrl+alt+space` as `Ctrl + Alt + Space`.
fn label(combo: &KeyCombo) -> String {
    combo
        .spelling()
        .split('+')
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bamboo_slip::Modifiers as Held;

    fn press(key: KeyCode, held: Held) -> KeyPress {
        KeyPress::new(key, held)
    }

    fn ctrl_alt() -> Held {
        Held {
            control: true,
            alt: true,
            ..Held::default()
        }
    }

    #[test]
    fn the_defaults_are_written_for_people() {
        let config = Config::default();
        assert_eq!(Shortcut::Toggle.current(&config), "Ctrl + Alt + Space");
        assert_eq!(Shortcut::Forget.current(&config), "Shift + Delete");
    }

    #[test]
    fn a_shortcut_needs_a_modifier_and_a_usable_key() {
        let config = Config::default();
        assert!(
            Shortcut::Toggle
                .accept(&press(KeyCode::Space, Held::default()), &config)
                .is_err()
        );
        assert!(
            Shortcut::Toggle
                .accept(&press(KeyCode::Escape, ctrl_alt()), &config)
                .is_err()
        );
        assert!(
            Shortcut::Toggle
                .accept(&press(KeyCode::Character("-".into()), ctrl_alt()), &config)
                .is_err()
        );
        assert_eq!(
            Shortcut::Toggle.accept(&press(KeyCode::Character("B".into()), ctrl_alt()), &config),
            Ok("ctrl+alt+b".to_owned())
        );
        assert_eq!(
            Shortcut::Forget.accept(
                &press(
                    KeyCode::Delete,
                    Held {
                        shift: true,
                        ..Held::default()
                    }
                ),
                &config
            ),
            Ok("shift+delete".to_owned())
        );
    }

    #[test]
    fn the_two_shortcuts_cannot_be_the_same() {
        let config = Config::default();
        let clash = Shortcut::Forget.accept(&press(KeyCode::Space, ctrl_alt()), &config);
        assert!(clash.unwrap_err().contains("Switch between"));
    }
}
