//! Counting words in corpus text and in count files.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use borno_dictionary::{normalize, tokens};

use crate::error::BuildError;
use crate::wikitext;

/// Word counts gathered from any number of inputs.
#[derive(Debug, Default)]
pub struct Counter {
    counts: HashMap<String, u64>,

    /// Bengali tokens seen, valid or not.
    pub tokens: u64,

    /// Tokens dropped as malformed.
    pub rejected: u64,

    /// Strip wiki markup from each line before counting.
    pub wikitext: bool,
}

impl Counter {
    /// A counter that strips wiki markup first when `wikitext` is set.
    pub fn new(wikitext: bool) -> Self {
        Self {
            wikitext,
            ..Self::default()
        }
    }

    /// Counts the Bengali words of one line of text.
    pub fn add_text(&mut self, text: &str) {
        if self.wikitext {
            let cleaned = wikitext::strip(text);
            return self.add_cleaned(&cleaned);
        }
        self.add_cleaned(text);
    }

    fn add_cleaned(&mut self, text: &str) {
        for token in tokens(text) {
            self.tokens += 1;
            match normalize(token) {
                Some(word) => *self.counts.entry(word).or_default() += 1,
                None => self.rejected += 1,
            }
        }
    }

    /// Counts everything readable from `reader`, line by line. Invalid UTF-8 is replaced, not fatal: a corpus
    /// is allowed to contain a few broken bytes.
    pub fn add_reader(&mut self, reader: impl Read) -> std::io::Result<()> {
        let mut reader = BufReader::new(reader);
        let mut line = Vec::new();
        while reader.read_until(b'\n', &mut line)? > 0 {
            self.add_text(&String::from_utf8_lossy(&line));
            line.clear();
        }
        Ok(())
    }

    /// Counts a file, a directory tree or (for `-`) standard input.
    pub fn add_path(&mut self, path: &Path) -> Result<(), BuildError> {
        let read = |source| BuildError::Read {
            path: path.to_owned(),
            source,
        };
        if path == Path::new("-") {
            return self.add_reader(std::io::stdin().lock()).map_err(read);
        }
        if path.is_dir() {
            let mut files: Vec<PathBuf> = Vec::new();
            collect_files(path, &mut files).map_err(read)?;
            files.sort();
            for file in files {
                self.add_path(&file)?;
            }
            return Ok(());
        }
        self.add_reader(File::open(path).map_err(read)?)
            .map_err(read)
    }

    /// Adds a `word<TAB>count` file. Words are normalized like corpus words; blank lines and `#` comments skip.
    pub fn add_counts(&mut self, path: &Path) -> Result<(), BuildError> {
        let file = File::open(path).map_err(|source| BuildError::Read {
            path: path.to_owned(),
            source,
        })?;
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line = line.map_err(|source| BuildError::Read {
                path: path.to_owned(),
                source,
            })?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let bad = || BuildError::BadCountLine {
                path: path.to_owned(),
                line: index + 1,
            };
            let (word, count) = line.split_once('\t').ok_or_else(bad)?;
            let count: u64 = count.trim().parse().map_err(|_| bad())?;
            self.tokens += 1;
            match normalize(word.trim()) {
                Some(word) => *self.counts.entry(word).or_default() += count,
                None => self.rejected += 1,
            }
        }
        Ok(())
    }

    /// Words seen at least `min_count` times, most frequent first, at most `max_words`.
    pub fn finish(self, min_count: u32, max_words: Option<usize>) -> Vec<(String, u32)> {
        let mut words: Vec<(String, u32)> = self
            .counts
            .into_iter()
            .filter(|(_, count)| *count >= u64::from(min_count))
            .map(|(word, count)| (word, u32::try_from(count).unwrap_or(u32::MAX)))
            .collect();
        words.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        if let Some(max) = max_words {
            words.truncate(max);
        }
        words
    }

    pub fn unique(&self) -> usize {
        self.counts.len()
    }
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_normalized_bengali_words_and_ignores_everything_else() {
        let mut counter = Counter::default();
        counter.add_text("আমি বাংলায় গান গাই। আমি tumi 2026 <ref>আমি</ref>\n");
        counter.add_text("ক্ ািম\n");
        assert_eq!(counter.rejected, 2);
        let words = counter.finish(1, None);
        assert_eq!(words[0], ("আমি".to_owned(), 3));
        assert_eq!(words.len(), 4);
    }

    #[test]
    fn min_count_and_max_words_trim_the_list_most_frequent_first() {
        let mut counter = Counter::default();
        counter.add_text("কক কক কক খখ খখ গগ");
        assert_eq!(counter.unique(), 3);
        let all = counter.finish(1, None);
        assert_eq!(all.iter().map(|(_, c)| *c).collect::<Vec<_>>(), [3, 2, 1]);
        let mut counter = Counter::default();
        counter.add_text("কক কক কক খখ খখ গগ");
        assert_eq!(counter.finish(2, None).len(), 2);
        let mut counter = Counter::default();
        counter.add_text("কক কক কক খখ খখ গগ");
        let top = counter.finish(1, Some(1));
        assert_eq!(top, [("কক".to_owned(), 3)]);
    }

    #[test]
    fn broken_utf8_is_replaced_not_fatal() {
        let mut counter = Counter::default();
        counter
            .add_reader(&b"\xff\xfe \xe0\xa6\x95\xe0\xa6\x95 \xff\n"[..])
            .unwrap();
        assert_eq!(counter.finish(1, None), [("কক".to_owned(), 1)]);
    }
}
