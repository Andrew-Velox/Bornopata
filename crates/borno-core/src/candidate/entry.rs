//! The candidate type.

use super::CandidateSource;

#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    /// The Bangla text that would be committed.
    pub text: String,

    /// Where the text came from; the highest-priority source when several produced it.
    pub source: CandidateSource,

    /// Ranking score, higher is better. Only comparable within one candidate list.
    pub score: f64,
}
