//! The language of the settings program's own text.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    /// Follow the system: Bangla when the locale is Bangla, English otherwise.
    #[default]
    #[serde(rename = "auto")]
    Auto,

    #[serde(rename = "en")]
    English,

    #[serde(rename = "bn")]
    Bangla,
}

impl Language {
    /// Whether the text should be Bangla: chosen, or for `Auto` whenever the locale says Bangla.
    pub fn uses_bangla(self) -> bool {
        match self {
            Self::Bangla => true,
            Self::English => false,
            Self::Auto => ["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"]
                .iter()
                .filter_map(|name| std::env::var(name).ok())
                .find(|value| !value.is_empty())
                .is_some_and(|value| locale_is_bangla(&value)),
        }
    }
}

/// Whether the first locale of a `LANG` or `LANGUAGE` style value is Bangla (`bn_BD.UTF-8`, `bn`, `bn:en`).
fn locale_is_bangla(value: &str) -> bool {
    let first = value.split(':').next().unwrap_or("");
    let language = first.split(['_', '.', '@']).next().unwrap_or("");
    language.eq_ignore_ascii_case("bn")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bangla_locales_are_recognised() {
        for value in ["bn_BD.UTF-8", "bn_IN", "bn", "BN_bd", "bn:en", "bn_BD@euro"] {
            assert!(locale_is_bangla(value), "{value}");
        }
        for value in ["en_US.UTF-8", "C", "", "en:bn", "bnx_XX"] {
            assert!(!locale_is_bangla(value), "{value}");
        }
    }

    #[test]
    fn a_chosen_language_ignores_the_locale() {
        assert!(Language::Bangla.uses_bangla());
        assert!(!Language::English.uses_bangla());
    }
}
