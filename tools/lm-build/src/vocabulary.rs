//! The words the model knows, numbered by falling frequency.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::BuildError;

/// Id 0 is the sentence start, the words begin at 1.
pub const SENTENCE_START_ID: u32 = 0;

pub struct Vocabulary {
    ids: HashMap<String, u32>,

    words: Vec<String>,
}

impl Vocabulary {
    /// The first `size` words of a `word<TAB>count` file.
    pub fn read(path: &Path, size: usize) -> Result<Self, BuildError> {
        let read = |source| BuildError::Read {
            path: path.to_owned(),
            source,
        };
        let mut vocabulary = Self {
            ids: HashMap::new(),
            words: Vec::new(),
        };
        for (index, line) in BufReader::new(File::open(path).map_err(read)?)
            .lines()
            .enumerate()
        {
            let line = line.map_err(read)?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (word, _) = line.split_once('\t').ok_or(BuildError::BadCountLine {
                path: path.to_owned(),
                line: index + 1,
            })?;
            if vocabulary.words.len() == size {
                break;
            }
            let id = vocabulary.words.len() as u32 + 1;
            if vocabulary.ids.insert(word.to_owned(), id).is_none() {
                vocabulary.words.push(word.to_owned());
            }
        }
        Ok(vocabulary)
    }

    pub fn id(&self, word: &str) -> Option<u32> {
        self.ids.get(word).copied()
    }

    /// The word with this id (not the sentence start).
    pub fn word(&self, id: u32) -> &str {
        &self.words[id as usize - 1]
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }
}
