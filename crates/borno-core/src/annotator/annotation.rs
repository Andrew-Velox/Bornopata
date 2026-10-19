//! What is shown beside a candidate.

/// A part of speech and a short text, such as `n.` and `rice`. One per candidate, in one auxiliary language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    /// Abbreviation such as `n.` or `v.`; empty when there is none.
    pub pos: String,

    /// At most two meanings, separated by a comma.
    pub text: String,
}
