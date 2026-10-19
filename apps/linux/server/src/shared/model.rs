//! The shared language model as an engine's own `LanguageModel`.

use std::sync::Arc;

use borno_core::LanguageModel;
use borno_lm::BigramModel;

pub(super) struct SharedModel(pub Arc<BigramModel>);

impl LanguageModel for SharedModel {
    fn log_prob(&self, previous: Option<&str>, word: &str) -> Option<f64> {
        self.0.log_prob(previous, word)
    }

    fn successors(&self, previous: Option<&str>, limit: usize) -> Vec<(String, f64)> {
        self.0.successors(previous, limit)
    }
}
