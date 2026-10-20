//! `WordSource` over the `upodesh` crate and its bundled word list.

use upodesh::avro::Suggest;

use super::WordSource;

/// The fallback word list bundled in `upodesh` (MIT; the original source of the words is not stated, which the project
/// accepted, see docs/design/bangla-reuse-plan.md).
pub struct UpodeshWordSource {
    suggest: Suggest,
}

impl UpodeshWordSource {
    pub fn new() -> Self {
        Self {
            suggest: Suggest::new(),
        }
    }
}

impl Default for UpodeshWordSource {
    fn default() -> Self {
        Self::new()
    }
}

impl WordSource for UpodeshWordSource {
    fn suggest(&self, roman: &str) -> Vec<String> {
        // upodesh builds its answer from hash maps, so the order changes between runs; sort for a stable answer.
        // The order therefore carries no ranking information (docs/design/bangla-engine.md).
        let mut words = self.suggest.suggest(roman);
        words.sort();
        words.dedup();
        words
    }
}
