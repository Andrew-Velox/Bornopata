//! The opened dictionary: exact and prefix lookup by phonetic key.

mod lookup;
mod open;

use borno_format::{Metadata, Table, Text};
use fst::Map;

use crate::layout::WordEntry;

/// A section of the mapped file, usable as the bytes of an FST.
pub(crate) struct Section(pub Table<u8>);

impl AsRef<[u8]> for Section {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// A word from the dictionary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Match<'a> {
    pub text: &'a str,

    /// How often the word occurred in the corpus.
    pub frequency: u32,

    /// The word's key is exactly the key asked for, not just an extension of it.
    pub exact: bool,
}

/// Words with frequencies behind a phonetic key index. Opening maps the file; nothing is copied.
pub struct Dictionary {
    keys: Map<Section>,
    members: Table<u32>,
    words: Text,
    entries: Table<WordEntry>,
    prefixes: Map<Section>,
    prefix_words: Table<u32>,
    metadata: Metadata,
}

impl Dictionary {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Where the data came from.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn word(&self, id: u32) -> &str {
        let entry = &self.entries[id as usize];
        let start = entry.text_start as usize;
        &self.words[start..start + usize::from(entry.text_len)]
    }

    fn matched(&self, id: u32, exact: bool) -> Match<'_> {
        Match {
            text: self.word(id),
            frequency: self.entries[id as usize].count,
            exact,
        }
    }
}

#[cfg(test)]
mod tests;
