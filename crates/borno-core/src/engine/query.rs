//! The result of a stateless candidate query.

use super::Timings;
use crate::candidate::Candidate;

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    /// The Roman input that was asked about.
    pub input: String,

    /// Candidates, best first.
    pub candidates: Vec<Candidate>,

    pub timings: Timings,
}
