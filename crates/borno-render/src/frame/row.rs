//! One row of the candidate window: index, word and annotation segments.

use super::{Source, Tone};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Index text shown before the word, such as `1`.
    pub index: String,

    /// The candidate word.
    pub text: String,

    /// Annotation segments, drawn in order after the word; empty when there is none.
    pub annotation: Vec<(String, Tone)>,

    /// Where the row came from.
    pub source: Source,
}

impl Row {
    /// A row with only an index and a word.
    pub fn plain(index: usize, text: impl Into<String>) -> Self {
        Self {
            index: (index + 1).to_string(),
            text: text.into(),
            annotation: Vec::new(),
            source: Source::Typed,
        }
    }
}
