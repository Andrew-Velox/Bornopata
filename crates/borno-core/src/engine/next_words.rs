//! Next-word suggestions after a committed word.

use std::collections::HashSet;

use super::Engine;

/// Spare successors asked of the language model so that hidden ones leave the list full.
const HIDDEN_ROOM: usize = 8;

impl Engine {
    /// Words likely after the last committed word: the user's own followers first, then the language model's.
    /// Meant for an empty composition; empty when nothing is known.
    pub fn next_words(&self, limit: usize) -> Vec<String> {
        let previous = self.previous.as_deref();
        let mut seen = HashSet::new();
        let mut words = Vec::new();
        let personal = self.learner.followers(previous, limit);
        // Ask for a few more than shown: the user may have hidden some of them.
        let model = self
            .language_model
            .successors(previous, limit + HIDDEN_ROOM)
            .into_iter()
            .map(|(word, _)| word)
            .filter(|word| !self.learner.is_suggestion_hidden(previous, word));
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
