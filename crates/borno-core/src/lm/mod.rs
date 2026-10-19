//! The seam to the word-level language model.

mod none;

pub use none::NoLanguageModel;

/// `log P(word | previous)` and likely successors. Implemented by `borno-lm`.
pub trait LanguageModel: Send {
    /// `previous` is `None` at the start of a sentence. `None` back means the model does not know `word`.
    fn log_prob(&self, previous: Option<&str>, word: &str) -> Option<f64>;

    /// Likely words after `previous`, most probable first, with their `log P`. Empty when the model has none.
    fn successors(&self, _previous: Option<&str>, _limit: usize) -> Vec<(String, f64)> {
        Vec::new()
    }
}
