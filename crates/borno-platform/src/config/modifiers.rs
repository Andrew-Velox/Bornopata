//! A set of modifier keys, written in the config as `ctrl+alt`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,

    /// The Super (Windows) key.
    pub super_key: bool,
}

impl Modifiers {
    pub fn is_empty(&self) -> bool {
        !(self.ctrl || self.alt || self.shift || self.super_key)
    }

    /// Config spelling in a fixed order: `ctrl+alt+shift+super`.
    pub fn spelling(&self) -> String {
        let mut parts = Vec::new();
        for (held, name) in [
            (self.ctrl, "ctrl"),
            (self.alt, "alt"),
            (self.shift, "shift"),
            (self.super_key, "super"),
        ] {
            if held {
                parts.push(name);
            }
        }
        parts.join("+")
    }
}

impl FromStr for Modifiers {
    type Err = String;

    /// Accepts `ctrl`/`control`, `alt`/`option`, `shift`, `super`/`win`/`cmd`/`meta`, joined by `+`, in any order.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut modifiers = Self::default();
        for token in text.split('+').map(str::trim).filter(|t| !t.is_empty()) {
            match token.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers.ctrl = true,
                "alt" | "option" => modifiers.alt = true,
                "shift" => modifiers.shift = true,
                "super" | "win" | "cmd" | "command" | "meta" => modifiers.super_key = true,
                other => return Err(format!("unknown modifier: {other}")),
            }
        }
        Ok(modifiers)
    }
}

impl TryFrom<String> for Modifiers {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        text.parse()
    }
}

impl From<Modifiers> for String {
    fn from(modifiers: Modifiers) -> Self {
        modifiers.spelling()
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.spelling())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_aliases_in_any_order_and_prints_canonically() {
        let parsed: Modifiers = "alt + Control".parse().unwrap();
        assert!(parsed.ctrl && parsed.alt && !parsed.shift);
        assert_eq!(parsed.spelling(), "ctrl+alt");
        assert_eq!(
            "win".parse::<Modifiers>().unwrap(),
            "super".parse().unwrap()
        );
        assert!("hyper".parse::<Modifiers>().is_err());
        assert!("".parse::<Modifiers>().unwrap().is_empty());
    }
}
