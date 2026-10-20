//! Where a candidate came from.

use std::fmt;

/// Variants are listed in priority order: a text produced by several sources carries the first that applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateSource {
    /// The user's own phrase for exactly this input.
    Phrase,

    /// Today's date or the time of day, from a code such as `tarikh`.
    Shortcut,

    /// The user chose this text for exactly this input before.
    Learned,

    /// The direct Avro conversion of the input.
    Direct,

    /// A word suggested by the word source as a spelling of the input.
    Dictionary,

    /// A word that only starts with what was typed (prefix completion).
    Completion,
}

impl fmt::Display for CandidateSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Phrase => "phrase",
            Self::Shortcut => "shortcut",
            Self::Learned => "learned",
            Self::Direct => "direct",
            Self::Dictionary => "dictionary",
            Self::Completion => "completion",
        })
    }
}
