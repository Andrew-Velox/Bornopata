//! Which entry to use when a word has several of different parts of speech.

use std::collections::HashMap;
use std::path::Path;

use crate::error::BuildError;

/// `word<TAB>part of speech` lines (Wiktextract's names: `conj`, `verb`, `noun`…), `#` for comments. A word listed
/// here is glossed from its entry of that part of speech first. This only chooses between Wiktionary's own entries; it
/// never supplies a gloss.
#[derive(Debug, Default)]
pub struct Prefer(HashMap<String, String>);

impl Prefer {
    pub fn parse(text: &str, path: &Path) -> Result<Self, BuildError> {
        let mut map = HashMap::new();
        for (number, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (word, pos) = line.split_once('\t').ok_or_else(|| BuildError::Line {
                path: path.to_owned(),
                line: number + 1,
                reason: "expected word<TAB>part of speech".to_owned(),
            })?;
            let word =
                borno_dictionary::normalize(word.trim()).ok_or_else(|| BuildError::Line {
                    path: path.to_owned(),
                    line: number + 1,
                    reason: format!("not a Bangla word: {word}"),
                })?;
            map.insert(word, pos.trim().to_owned());
        }
        Ok(Self(map))
    }

    pub fn load(path: &Path) -> Result<Self, BuildError> {
        let text = std::fs::read_to_string(path).map_err(|source| BuildError::Read {
            path: path.to_owned(),
            source,
        })?;
        Self::parse(&text, path)
    }

    pub fn pos_of(&self, word: &str) -> Option<&str> {
        self.0.get(word).map(String::as_str)
    }
}
