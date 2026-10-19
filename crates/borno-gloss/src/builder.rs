//! Collecting entries and writing the `.bnd` file.

use std::collections::BTreeMap;
use std::path::Path;

use borno_dictionary::normalize;
use borno_format::{Kind, Metadata, Table, Writer, hash};

use crate::error::GlossaryError;
use crate::record::{ENTRIES_TAG, EntryRecord, HASH_TAG, TEXT_TAG};
use crate::{MAX_POS_BYTES, MAX_TEXT_BYTES};

/// Builds a glossary. Words are stored in their canonical spelling, so a lookup with any spelling of the same word
/// finds it. A word added twice keeps the last gloss.
#[derive(Debug, Default)]
pub struct GlossaryBuilder {
    entries: BTreeMap<String, (String, String)>,
}

impl GlossaryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `word` with its part of speech (may be empty) and text. Fails on a malformed word, an empty or too long
    /// text, or a part of speech that is too long.
    pub fn add(&mut self, word: &str, pos: &str, text: &str) -> Result<(), GlossaryError> {
        let word =
            normalize(word).ok_or_else(|| line_error(format!("not a Bangla word: {word:?}")))?;
        let (pos, text) = (pos.trim(), text.trim());
        if text.is_empty() || text.len() > MAX_TEXT_BYTES {
            return Err(line_error(format!(
                "gloss text of {word} is empty or longer than {MAX_TEXT_BYTES} bytes"
            )));
        }
        if pos.len() > MAX_POS_BYTES || text.contains(['\t', '\n']) || pos.contains(['\t', '\n']) {
            return Err(line_error(format!(
                "part of speech or text of {word} is malformed"
            )));
        }
        self.entries.insert(word, (pos.to_owned(), text.to_owned()));
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Writes the file (to a temporary name, then renamed, so a reader never sees half a file).
    pub fn write(&self, path: &Path, metadata: &Metadata) -> Result<(), GlossaryError> {
        let mut arena = String::new();
        let mut records = Vec::with_capacity(self.entries.len());
        let mut push = |piece: &str| -> (u32, u16) {
            let start = arena.len() as u32;
            arena.push_str(piece);
            (start, piece.len() as u16)
        };
        for (word, (pos, text)) in &self.entries {
            let (word_start, word_len) = push(word);
            let (pos_start, pos_len) = push(pos);
            let (text_start, text_len) = push(text);
            records.push(EntryRecord {
                word_start,
                pos_start,
                text_start,
                word_len,
                pos_len,
                text_len,
                reserved: 0,
            });
        }
        let index = hash::build(records.len(), |id| {
            let record = &records[id as usize];
            &arena[record.word_start as usize
                ..record.word_start as usize + usize::from(record.word_len)]
        });
        let metadata = Metadata {
            entries: records.len() as u64,
            ..metadata.clone()
        };
        let (records, index) = (Table::Owned(records), Table::Owned(index));
        Writer::new(Kind::Glossary, &metadata)?
            .section(TEXT_TAG, arena.as_bytes())
            .section(ENTRIES_TAG, records.as_bytes())
            .section(HASH_TAG, index.as_bytes())
            .write_to(path)?;
        Ok(())
    }
}

fn line_error(reason: String) -> GlossaryError {
    GlossaryError::Line { line: 0, reason }
}
