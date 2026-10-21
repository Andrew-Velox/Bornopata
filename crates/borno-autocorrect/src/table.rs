//! The correction table and how a typed word finds its row.

use std::collections::HashMap;

use borno_core::AutocorrectSource;

use crate::suffix;

/// The table from `assets/autocorrect/autocorrect.tsv`.
const BUNDLED: &str = include_str!("../../../assets/autocorrect/autocorrect.tsv");

/// English words and the Bangla spellings they get. Never changes after it is built, so engines share one behind an
/// `Arc`.
pub struct AutocorrectTable {
    words: HashMap<String, Vec<String>>,
}

impl AutocorrectTable {
    /// The table that ships with the program.
    pub fn bundled() -> Self {
        Self::from_tsv(BUNDLED)
    }

    /// Reads `word<TAB>Bangla` rows. Rows with one word give several spellings, in order. Comment lines start with `#`;
    /// a bad row is skipped.
    pub fn from_tsv(text: &str) -> Self {
        let mut words: HashMap<String, Vec<String>> = HashMap::new();
        for line in text.lines() {
            if line.starts_with('#') {
                continue;
            }
            let Some((word, bangla)) = line.split_once('\t') else {
                continue;
            };
            if word.is_empty() || bangla.is_empty() || !word.bytes().all(|b| b.is_ascii_lowercase())
            {
                continue;
            }
            let spellings = words.entry(word.to_owned()).or_default();
            if !spellings.iter().any(|known| known == bangla) {
                spellings.push(bangla.to_owned());
            }
        }
        Self { words }
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

impl AutocorrectSource for AutocorrectTable {
    fn lookup(&self, input: &str) -> Vec<String> {
        let Some(word) = as_written(input) else {
            return Vec::new();
        };
        if let Some(spellings) = self.words.get(&word) {
            return spellings.clone();
        }
        for (stem, ending) in suffix::split(&word) {
            let Some(spellings) = self.words.get(stem) else {
                continue;
            };
            let found: Vec<String> = spellings
                .iter()
                .filter(|spelling| suffix::fits(spelling, ending))
                .map(|spelling| format!("{spelling}{ending}"))
                .collect();
            if !found.is_empty() {
                return found;
            }
        }
        Vec::new()
    }
}

/// `input` in lower case when it reads as an ordinary written word: all lower case, capitalized (`Facebook`) or all
/// capitals (`TV`). Any other mix of capitals is Avro spelling (`ShikkhA`) and is left alone.
fn as_written(input: &str) -> Option<String> {
    if input.is_empty() || !input.bytes().all(|b| b.is_ascii_alphabetic()) {
        return None;
    }
    let rest = &input[1..];
    let ordinary = rest.bytes().all(|b| b.is_ascii_lowercase())
        || input.bytes().all(|b| b.is_ascii_uppercase());
    ordinary.then(|| input.to_ascii_lowercase())
}
