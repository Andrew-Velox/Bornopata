//! The mapped glossary: lookup by word.

mod open;

use borno_core::{Annotation, Annotator};
use borno_dictionary::normalize;
use borno_format::{Metadata, Table, Text, hash};

use crate::gloss::Gloss;
use crate::record::EntryRecord;

/// A glossary mapped from a `.bnd` file. Shared between input contexts behind an `Arc`; it never changes.
#[derive(Debug)]
pub struct Glossary {
    text: Text,
    entries: Table<EntryRecord>,
    index: Table<u32>,
    metadata: Metadata,
}

impl Glossary {
    /// The gloss of `word` (any spelling of it), if the glossary has one.
    pub fn lookup(&self, word: &str) -> Option<Gloss<'_>> {
        let word = normalize(word)?;
        let id = hash::find(&self.index, &word, |id| self.word(id))?;
        let record = self.entries.get(id as usize)?;
        Some(Gloss {
            pos: self.slice(record.pos_start, record.pos_len),
            text: self.slice(record.text_start, record.text_len),
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Where the data came from: name, license, attribution.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn word(&self, id: u32) -> &str {
        let record = &self.entries[id as usize];
        self.slice(record.word_start, record.word_len)
    }

    /// Every range was checked to lie inside the arena on a character boundary when the file was opened.
    fn slice(&self, start: u32, len: u16) -> &str {
        &self.text[start as usize..start as usize + usize::from(len)]
    }
}

impl Annotator for Glossary {
    fn annotate(&self, word: &str) -> Option<Annotation> {
        let gloss = self.lookup(word)?;
        Some(Annotation {
            pos: gloss.pos.to_owned(),
            text: gloss.text.to_owned(),
        })
    }
}
