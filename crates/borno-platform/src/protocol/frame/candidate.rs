//! One line of the candidate list.

use serde::{Deserialize, Serialize};

use super::Source;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameCandidate {
    pub text: String,

    pub source: Source,
}
