//! One line of the candidate list.

use serde::{Deserialize, Serialize};

use super::{FrameGloss, Source};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameCandidate {
    pub text: String,

    pub source: Source,

    /// The short gloss for this word, when the Server has one and glosses are turned on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gloss: Option<FrameGloss>,
}
