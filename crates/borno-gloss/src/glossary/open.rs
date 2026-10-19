//! Opening and validating the file.

use std::path::Path;

use borno_format::{Container, Kind, hash};

use super::Glossary;
use crate::error::GlossaryError;
use crate::record::{ENTRIES_TAG, EntryRecord, HASH_TAG, TEXT_TAG};

impl Glossary {
    /// Maps `path` and checks every range lies inside the arena on character boundaries and the hash index matches the
    /// entries, so a lookup cannot index out of bounds or loop on a damaged file.
    pub fn open(path: &Path) -> Result<Self, GlossaryError> {
        let container = Container::open(path, Kind::Glossary)?;
        let glossary = Self {
            text: container.text(TEXT_TAG)?,
            entries: container.table(ENTRIES_TAG)?,
            index: container.table(HASH_TAG)?,
            metadata: container.metadata().clone(),
        };
        glossary.validate()?;
        tracing::debug!(entries = glossary.len(), name = %glossary.metadata.name, "glossary mapped");
        Ok(glossary)
    }

    fn validate(&self) -> Result<(), GlossaryError> {
        let inside = |start: u32, len: u16| {
            self.text
                .get(start as usize..start as usize + usize::from(len))
                .is_some()
        };
        for record in self.entries.iter() {
            let EntryRecord {
                word_start,
                pos_start,
                text_start,
                word_len,
                pos_len,
                text_len,
                ..
            } = *record;
            if !inside(word_start, word_len)
                || !inside(pos_start, pos_len)
                || !inside(text_start, text_len)
            {
                return Err(GlossaryError::Corrupt("an entry points outside the text"));
            }
        }
        if !hash::is_valid(&self.index, self.entries.len()) {
            return Err(GlossaryError::Corrupt(
                "the hash index does not match the entries",
            ));
        }
        Ok(())
    }
}
