//! The most frequent words of a `word<TAB>count` file.

use std::collections::HashMap;
use std::path::Path;

use crate::error::BuildError;

/// Counts by word, from lines `word<TAB>count`.
pub fn read_counts(path: &Path) -> Result<HashMap<String, u64>, BuildError> {
    let text = std::fs::read_to_string(path).map_err(|source| BuildError::Read {
        path: path.to_owned(),
        source,
    })?;
    let mut counts = HashMap::new();
    for (number, raw) in text.lines().enumerate() {
        if raw.trim().is_empty() {
            continue;
        }
        let bad = |reason: &str| BuildError::Line {
            path: path.to_owned(),
            line: number + 1,
            reason: reason.to_owned(),
        };
        let (word, count) = raw
            .split_once('\t')
            .ok_or_else(|| bad("expected word<TAB>count"))?;
        let count: u64 = count
            .trim()
            .parse()
            .map_err(|_| bad("the count is not a number"))?;
        *counts.entry(word.to_owned()).or_insert(0) += count;
    }
    Ok(counts)
}

/// The `limit` most frequent words, most frequent first; equal counts in code point order.
pub fn top(counts: &HashMap<String, u64>, limit: usize) -> Vec<(String, u64)> {
    let mut words: Vec<(String, u64)> = counts
        .iter()
        .map(|(word, count)| (word.clone(), *count))
        .collect();
    words.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    words.truncate(limit);
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_most_frequent_words_come_first_and_ties_are_ordered_by_spelling() {
        let counts = HashMap::from([
            ("খ".to_owned(), 5),
            ("ক".to_owned(), 5),
            ("গ".to_owned(), 9),
            ("ঘ".to_owned(), 1),
        ]);
        let words: Vec<String> = top(&counts, 3).into_iter().map(|(w, _)| w).collect();
        assert_eq!(words, ["গ", "ক", "খ"]);
    }
}
