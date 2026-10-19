//! Where a candidate came from.

use std::fmt;

/// Variants are listed in priority order: a text produced by several sources carries the first that applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateSource {
    /// The user chose this text for exactly this input before.
    Learned,

    /// The direct Avro conversion of the input.
    Direct,

    /// A word suggested by the word source.
    Dictionary,
}

impl fmt::Display for CandidateSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Learned => "learned",
            Self::Direct => "direct",
            Self::Dictionary => "dictionary",
        })
    }
}
