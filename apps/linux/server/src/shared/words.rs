//! The shared word source as an engine's own `WordSource`.

use std::sync::Arc;

use borno_transliteration::WordSource;

pub(super) struct SharedWords(pub Arc<dyn WordSource + Send + Sync>);

impl WordSource for SharedWords {
    fn suggest(&self, roman: &str) -> Vec<String> {
        self.0.suggest(roman)
    }
}
