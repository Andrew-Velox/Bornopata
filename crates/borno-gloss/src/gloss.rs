//! One gloss, borrowed from the mapped file.

/// What is shown beside a candidate: a part of speech and the meaning, such as `n.` and `rice`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gloss<'a> {
    /// Abbreviation such as `n.` or `v.`; empty when the data has none.
    pub pos: &'a str,

    /// At most two meanings, separated by a comma: `group, team`.
    pub text: &'a str,
}
