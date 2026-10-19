//! `WordSource` over the `upodesh` crate and its bundled word list.

use upodesh::avro::Suggest;

use super::WordSource;

/// Development scaffold only: the word list bundled in `upodesh` has unverified provenance
/// and must be replaced before release (docs/design/bangla-reuse-plan.md).
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
        self.suggest.suggest(roman)
    }
}
