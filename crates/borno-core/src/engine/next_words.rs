//! Next-word suggestions after a committed word.

use std::collections::HashSet;

use super::Engine;

impl Engine {
    /// Words likely after the last committed word: the user's own followers first, then the language model's.
    /// Meant for an empty composition; empty when nothing is known.
    pub fn next_words(&self, limit: usize) -> Vec<String> {
        let previous = self.previous.as_deref();
        let mut seen = HashSet::new();
        let mut words = Vec::new();
        let personal = self.learner.followers(previous, limit);
        let model = self
            .language_model
            .successors(previous, limit)
            .into_iter()
            .map(|(word, _)| word);
        for word in personal.into_iter().map(|(word, _)| word).chain(model) {
            if words.len() == limit {
                break;
            }
            if seen.insert(word.clone()) {
                words.push(word);
            }
        }
        words
    }
}
