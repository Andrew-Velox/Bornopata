//! The pages of the sidebar, in order.

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

    pub fn label(self) -> &'static str {
        match self {
            Self::Typing => "Typing",
            Self::Keys => "Keys",
            Self::Appearance => "Appearance",
            Self::Phrases => "Phrases",
            Self::Learning => "Learning",
            Self::About => "About",
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
