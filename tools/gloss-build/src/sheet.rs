//! The review sheet: the most frequent words with their gloss next to what Wiktionary says, for a person to check.

use std::collections::HashMap;
use std::path::Path;

use crate::error::BuildError;
use crate::kaikki::Index;
use crate::ranks::top;
use crate::table::Entry;

/// Writes `rows` words, most frequent first, as a TSV: rank, word, count, part of speech, gloss, source, and the first
/// sense of the first Wiktionary entry. Words without a gloss are listed too, so gaps are visible.
pub fn write(
    path: &Path,
    rows: usize,
    counts: &HashMap<String, u64>,
    entries: &[Entry],
    index: Option<&Index>,
) -> Result<(), BuildError> {
    let by_word: HashMap<&str, &Entry> = entries
        .iter()
        .map(|entry| (entry.word.as_str(), entry))
        .collect();
    let mut out = String::from(
        "rank\tword\tcount\tpos\tgloss\tsource\twiktionary (first sense)\tyour verdict\n",
    );
    for (rank, (word, count)) in top(counts, rows).iter().enumerate() {
        let (pos, text, source) = by_word
            .get(word.as_str())
            .map_or(("", "", "(none)"), |entry| {
                (&entry.pos, &entry.text, &entry.source)
            });
        let raw = index
            .map(|index| first_sense(index, word))
            .unwrap_or_default();
        out.push_str(&format!(
            "{}\t{word}\t{count}\t{pos}\t{text}\t{source}\t{raw}\t\n",
            rank + 1
        ));
    }
    std::fs::write(path, out).map_err(|source| BuildError::Write {
        path: path.to_owned(),
        source,
    })
}

fn first_sense(index: &Index, word: &str) -> String {
    index
        .lines(word)
        .iter()
        .find_map(|line| {
            let gloss = line.senses.first()?.gloss()?;
            Some(format!(
                "{}: {}",
                line.pos,
                gloss.chars().take(70).collect::<String>()
            ))
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_ranked_word_has_a_row_and_a_missing_gloss_is_marked() {
        let counts = HashMap::from([("ভাত".to_owned(), 9), ("পানি".to_owned(), 5)]);
        let entries = [Entry {
            word: "ভাত".into(),
            pos: "n.".into(),
            text: "rice".into(),
            source: "wiktionary".into(),
        }];
        let path =
            std::env::temp_dir().join(format!("borno-gloss-sheet-{}.tsv", std::process::id()));
        write(&path, 5, &counts, &entries, None).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let rows: Vec<&str> = text.lines().collect();
        assert_eq!(rows.len(), 3);
        assert!(rows[1].starts_with("1\tভাত\t9\tn.\trice\twiktionary"));
        assert!(rows[2].starts_with("2\tপানি\t5\t\t\t(none)"));
        std::fs::remove_file(path).unwrap();
    }
}
