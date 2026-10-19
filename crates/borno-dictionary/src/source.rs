//! The dictionary as a word source: typed Roman in, ranked Bangla words out.

use std::path::Path;

use borno_transliteration::{
    FallbackWordSource, Transliterator, UpodeshWordSource, WordSource, WordSuggestion,
};

use crate::dictionary::Dictionary;
use crate::error::DictionaryError;
use crate::key::key;

/// Looks the typed Roman up by its phonetic key: the Avro conversion of the input, reduced to its key
/// (see [`crate::key`]), finds the words people mean by it and, further down, the words it is the start of.
pub struct DictionarySource {
    dictionary: Dictionary,
    transliterator: Transliterator,
}

impl DictionarySource {
    pub fn new(dictionary: Dictionary) -> Self {
        Self {
            dictionary,
            transliterator: Transliterator::new(),
        }
    }

    pub fn open(path: &Path) -> Result<Self, DictionaryError> {
        Ok(Self::new(Dictionary::open(path)?))
    }

    pub fn dictionary(&self) -> &Dictionary {
        &self.dictionary
    }
}

/// The word source the programs use: the dictionary at `path` with the `upodesh` scaffold behind it as a fallback,
/// or the scaffold alone when there is no dictionary.
pub fn word_source(
    path: Option<&Path>,
) -> Result<Box<dyn WordSource + Send + Sync>, DictionaryError> {
    let scaffold = Box::new(UpodeshWordSource::new());
    match path {
        Some(path) => Ok(Box::new(FallbackWordSource::new(
            Box::new(DictionarySource::open(path)?),
            scaffold,
        ))),
        None => Ok(scaffold),
    }
}

impl WordSource for DictionarySource {
    fn suggest(&self, roman: &str) -> Vec<String> {
        self.lookup(roman, usize::MAX)
            .into_iter()
            .filter(|suggestion| !suggestion.completion)
            .map(|suggestion| suggestion.text)
            .collect()
    }

    fn lookup(&self, roman: &str, limit: usize) -> Vec<WordSuggestion> {
        let typed = self.transliterator.convert(roman);
        self.dictionary
            .lookup(&key(&typed), limit)
            .into_iter()
            .map(|found| WordSuggestion {
                text: found.text.to_owned(),
                frequency: found.frequency,
                completion: !found.exact,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use borno_format::Metadata;

    use super::*;
    use crate::DictionaryBuilder;

    fn source(words: &[(&str, u32)]) -> DictionarySource {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let number = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("borno-source-{}-{number}.bnd", std::process::id()));
        let mut builder = DictionaryBuilder::new();
        for (word, count) in words {
            builder.add(word, *count);
        }
        builder.write(&path, &Metadata::default()).unwrap();
        let source = DictionarySource::open(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        source
    }

    #[test]
    fn typed_variants_find_the_same_word() {
        let source = source(&[("কথা", 500), ("কোথা", 900), ("আমি", 1000)]);
        for typed in ["kotha", "kOtha", "kothaa", "Kotha"] {
            let found: Vec<_> = source
                .lookup(typed, 5)
                .into_iter()
                .map(|s| s.text)
                .collect();
            assert_eq!(found, ["কোথা", "কথা"], "{typed}");
        }
        assert_eq!(source.suggest("amii"), ["আমি"]);
    }

    #[test]
    fn a_prefix_completes_and_marks_the_completions() {
        let source = source(&[("বাংলা", 400), ("বাংলাদেশ", 450), ("বাংলাদেশী", 100)]);
        let found = source.lookup("bangl", 5);
        assert!(found.iter().all(|s| s.completion));
        assert_eq!(found[0].text, "বাংলাদেশ");
        let found = source.lookup("bangla", 5);
        assert_eq!(
            (found[0].text.as_str(), found[0].completion),
            ("বাংলা", false)
        );
        assert!(found[1..].iter().all(|s| s.completion));
        assert_eq!(source.suggest("bangla"), ["বাংলা"]);
    }

    #[test]
    fn unknown_and_empty_input_find_nothing() {
        let source = source(&[("আমি", 1)]);
        assert!(source.lookup("", 5).is_empty());
        assert!(source.lookup("zzzzzz", 5).is_empty());
        assert!(source.lookup("…", 5).is_empty());
    }
}
