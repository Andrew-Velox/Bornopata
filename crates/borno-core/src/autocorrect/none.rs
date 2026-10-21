//! An autocorrect source with no corrections.

use super::AutocorrectSource;

#[derive(Debug, Clone, Copy, Default)]
pub struct NoAutocorrect;

impl AutocorrectSource for NoAutocorrect {
    fn lookup(&self, _input: &str) -> Vec<String> {
        Vec::new()
    }
}
