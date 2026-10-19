//! What the shell should show for one session.

mod candidate;
mod source;

pub use candidate::FrameCandidate;
pub use source::Source;

use serde::{Deserialize, Serialize};

use super::Mode;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    /// The Roman letters typed for the current word.
    pub roman: String,

    /// The Bangla text of the highlighted candidate, shown in place of the Roman letters.
    pub preedit: String,

    /// At most nine, best first.
    pub candidates: Vec<FrameCandidate>,

    /// Index of the highlighted candidate.
    pub highlight: usize,

    /// The candidates are next-word suggestions after a committed word, not completions of `roman`.
    pub suggesting: bool,

    pub mode: Mode,
}

impl Frame {
    /// Nothing to show: the shell hides its panel.
    pub fn is_empty(&self) -> bool {
        self.roman.is_empty() && self.candidates.is_empty()
    }
}
