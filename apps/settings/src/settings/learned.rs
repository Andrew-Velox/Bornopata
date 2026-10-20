//! How much Bornopata has learned, read from the learner's files.

use std::path::Path;

use borno_platform::paths;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Learned {
    /// Distinct words that were ever committed.
    pub words: usize,

    /// Which word was chosen for which letters.
    pub choices: usize,

    /// Which word followed which.
    pub pairs: usize,
}

impl Learned {
    pub fn load() -> Self {
        let dir = paths::user_dir();
        Self {
            words: rows(&dir.join("user-words.tsv")),
            choices: rows(&dir.join("user-choices.tsv")),
            pairs: rows(&dir.join("user-ngram.tsv")),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.words + self.choices + self.pairs == 0
    }
}

/// Lines that hold data: the files have no header, only comments and blanks to skip.
fn rows(path: &Path) -> usize {
    std::fs::read_to_string(path)
        .map(|text| {
            text.lines()
                .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                .count()
        })
        .unwrap_or(0)
}
