//! In-memory `WordSource` for tests and for swapping in a different list.

use super::WordSource;

/// Roman input mapped to its words by exact match, in the order given.
pub struct FixedWordSource {
    entries: Vec<(String, Vec<String>)>,
}

impl FixedWordSource {
    pub fn new(entries: impl IntoIterator<Item = (String, Vec<String>)>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }
}

impl WordSource for FixedWordSource {
    fn suggest(&self, roman: &str) -> Vec<String> {
        self.entries
            .iter()
            .find(|(key, _)| key == roman)
            .map(|(_, words)| words.clone())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_words_for_an_exact_key_and_nothing_otherwise() {
        let source = FixedWordSource::new([("ami".to_owned(), vec!["আমি".to_owned()])]);
        assert_eq!(source.suggest("ami"), ["আমি"]);
        assert!(source.suggest("tumi").is_empty());
    }
}
