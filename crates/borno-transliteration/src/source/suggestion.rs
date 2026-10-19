//! A word offered by a word source.

/// One answer of [`super::WordSource::lookup`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordSuggestion {
    pub text: String,

    /// How often the word occurs in the source's corpus; 0 when the source does not know.
    pub frequency: u32,

    /// The word merely starts with what was typed (prefix completion) instead of being a spelling of it.
    pub completion: bool,
}

impl WordSuggestion {
    /// A spelling of the typed input, with no frequency.
    pub fn plain(text: String) -> Self {
        Self {
            text,
            frequency: 0,
            completion: false,
        }
    }
}
