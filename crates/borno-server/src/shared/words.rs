//! The shared word source as an engine's own `WordSource`.

use std::sync::Arc;

use borno_transliteration::{WordSource, WordSuggestion};

pub(super) struct SharedWords(pub Arc<dyn WordSource + Send + Sync>);

impl WordSource for SharedWords {
    fn suggest(&self, roman: &str) -> Vec<String> {
        self.0.suggest(roman)
    }

    /// Forwarded explicitly: the default `lookup` would drop the frequencies and the completions.
    fn lookup(&self, roman: &str, limit: usize) -> Vec<WordSuggestion> {
        self.0.lookup(roman, limit)
    }
}
