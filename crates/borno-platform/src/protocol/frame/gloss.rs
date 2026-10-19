//! The gloss shown beside a candidate.

use serde::{Deserialize, Serialize};

/// A part of speech and a short text in the auxiliary language, such as `n.` and `rice`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameGloss {
    /// Abbreviation such as `n.` or `v.`; empty when there is none.
    pub pos: String,

    /// At most two meanings, separated by a comma.
    pub text: String,
}
