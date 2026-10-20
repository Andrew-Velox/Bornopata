//! A phrase source with no phrases.

use super::PhraseSource;

#[derive(Debug, Clone, Copy, Default)]
pub struct NoPhrases;

impl PhraseSource for NoPhrases {
    fn lookup(&self, _code: &str) -> Vec<String> {
        Vec::new()
    }
}
