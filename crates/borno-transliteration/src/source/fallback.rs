//! A word source that asks a second one only when the first has no spelling of the input.

use super::{WordSource, WordSuggestion};

/// The dictionary answers first. The fallback (the `upodesh` scaffold) is consulted only when the dictionary has
/// no word that spells the input, so its unranked answers never mix into a list the dictionary can rank.
pub struct FallbackWordSource {
    primary: Box<dyn WordSource + Send + Sync>,
    fallback: Box<dyn WordSource + Send + Sync>,
}

impl FallbackWordSource {
    pub fn new(
        primary: Box<dyn WordSource + Send + Sync>,
        fallback: Box<dyn WordSource + Send + Sync>,
    ) -> Self {
        Self { primary, fallback }
    }
}

impl WordSource for FallbackWordSource {
    fn suggest(&self, roman: &str) -> Vec<String> {
        self.lookup(roman, usize::MAX)
            .into_iter()
            .map(|suggestion| suggestion.text)
            .collect()
    }

    fn lookup(&self, roman: &str, limit: usize) -> Vec<WordSuggestion> {
        let mut found = self.primary.lookup(roman, limit);
        if found.iter().any(|suggestion| !suggestion.completion) {
            return found;
        }
        for suggestion in self.fallback.lookup(roman, limit) {
            if found.len() >= limit {
                break;
            }
            if !found.iter().any(|known| known.text == suggestion.text) {
                found.push(suggestion);
            }
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FixedWordSource;

    fn fixed(entries: &[(&str, &[&str])]) -> Box<dyn WordSource + Send + Sync> {
        Box::new(FixedWordSource::new(entries.iter().map(
            |(roman, words)| {
                (
                    (*roman).to_owned(),
                    words.iter().map(|w| (*w).to_owned()).collect(),
                )
            },
        )))
    }

    #[test]
    fn the_fallback_is_ignored_while_the_primary_has_a_spelling() {
        let source =
            FallbackWordSource::new(fixed(&[("ami", &["আমি"])]), fixed(&[("ami", &["অমি"])]));
        let texts: Vec<_> = source
            .lookup("ami", 5)
            .into_iter()
            .map(|s| s.text)
            .collect();
        assert_eq!(texts, ["আমি"]);
    }

    #[test]
    fn the_fallback_fills_in_when_the_primary_knows_nothing() {
        let source = FallbackWordSource::new(fixed(&[]), fixed(&[("tumi", &["তুমি", "তূমি"])]));
        let texts: Vec<_> = source
            .lookup("tumi", 5)
            .into_iter()
            .map(|s| s.text)
            .collect();
        assert_eq!(texts, ["তুমি", "তূমি"]);
        assert_eq!(source.lookup("tumi", 1).len(), 1);
        assert_eq!(source.suggest("tumi"), ["তুমি", "তূমি"]);
        assert!(source.lookup("zzz", 5).is_empty());
    }
}
