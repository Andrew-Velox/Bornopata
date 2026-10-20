//! The pages of the sidebar, in order.

use crate::i18n::t;
use crate::view::icons::Glyph;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageId {
    Typing,
    Keys,
    Appearance,
    Phrases,
    Learning,
    About,
}

impl PageId {
    pub const ALL: [Self; 6] = [
        Self::Typing,
        Self::Keys,
        Self::Appearance,
        Self::Phrases,
        Self::Learning,
        Self::About,
    ];

    /// The page called `name` (`typing`, `keys`, ...), for `--page`.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|page| format!("{page:?}").eq_ignore_ascii_case(name))
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Typing => t("Typing"),
            Self::Keys => t("Keys"),
            Self::Appearance => t("Appearance"),
            Self::Phrases => t("Phrases"),
            Self::Learning => t("Learning"),
            Self::About => t("About"),
        }
    }

    pub fn glyph(self) -> Glyph {
        match self {
            Self::Typing => Glyph::Keyboard,
            Self::Keys => Glyph::Command,
            Self::Appearance => Glyph::Appearance,
            Self::Phrases => Glyph::Phrases,
            Self::Learning => Glyph::Learning,
            Self::About => Glyph::About,
        }
    }
}
