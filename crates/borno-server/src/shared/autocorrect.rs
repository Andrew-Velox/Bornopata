//! The shared autocorrect table as an engine's own `AutocorrectSource`.

use std::sync::Arc;

use borno_autocorrect::AutocorrectTable;
use borno_core::AutocorrectSource;

pub(super) struct SharedAutocorrect(pub Arc<AutocorrectTable>);

impl AutocorrectSource for SharedAutocorrect {
    fn lookup(&self, input: &str) -> Vec<String> {
        self.0.lookup(input)
    }
}
