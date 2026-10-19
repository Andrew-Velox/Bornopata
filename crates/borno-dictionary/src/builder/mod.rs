//! Building the dictionary file from counted words.

mod prefixes;

use std::collections::BTreeMap;
use std::path::Path;

use borno_format::{Kind, Metadata, Writer};
use fst::MapBuilder;
use zerocopy::IntoBytes;

use crate::error::DictionaryError;
use crate::key::key;
use crate::layout::{
    ENTRIES_TAG, Group, KEYS_TAG, MEMBERS_TAG, PREFIX_WORDS_TAG, PREFIXES_TAG, WORDS_TAG, WordEntry,
};

/// Collects (word, count) pairs and writes the `.bnd` file. Words must already be normalized.
#[derive(Debug, Default)]
pub struct DictionaryBuilder {
    words: Vec<(String, u32)>,
}

impl DictionaryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a word seen `count` times. The same word added twice is counted once with the sum.
    pub fn add(&mut self, word: &str, count: u32) {
        if !word.is_empty() && count > 0 {
            self.words.push((word.to_owned(), count));
        }
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Writes the dictionary. `metadata.entries` is filled in.
    pub fn write(self, path: &Path, metadata: &Metadata) -> Result<(), DictionaryError> {
        let words = self.ranked();
        let metadata = Metadata {
            entries: words.len() as u64,
            ..metadata.clone()
        };

        let mut arena = String::new();
        let mut entries = Vec::with_capacity(words.len());
        for (word, count) in &words {
            entries.push(WordEntry {
                text_start: u32::try_from(arena.len())
                    .map_err(|_| DictionaryError::Corrupt("word text over 4 GiB"))?,
                count: *count,
                text_len: u16::try_from(word.len())
                    .map_err(|_| DictionaryError::Corrupt("word longer than 64 KiB"))?,
                reserved: 0,
            });
            arena.push_str(word);
        }

        // Ids ascend while groups are filled, so each group is ranked by frequency.
        let mut groups: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        for (id, (word, _)) in words.iter().enumerate() {
            groups.entry(key(word)).or_default().push(id as u32);
        }
        let mut members: Vec<u32> = Vec::with_capacity(words.len());
        let mut keys = MapBuilder::memory();
        for (group_key, ids) in &groups {
            let ids = &ids[..ids.len().min(Group::MAX_COUNT)];
            let group = Group {
                start: members.len(),
                count: ids.len(),
            };
            members.extend_from_slice(ids);
            keys.insert(group_key.as_bytes(), group.to_value())?;
        }

        let (prefix_index, prefix_words) = prefixes::build(&groups)?;
        Writer::new(Kind::Dictionary, &metadata)?
            .section(KEYS_TAG, &keys.into_inner()?)
            .section(MEMBERS_TAG, members.as_bytes())
            .section(WORDS_TAG, arena.as_bytes())
            .section(ENTRIES_TAG, entries.as_bytes())
            .section(PREFIXES_TAG, &prefix_index)
            .section(PREFIX_WORDS_TAG, prefix_words.as_bytes())
            .write_to(path)?;
        Ok(())
    }

    /// The words merged by spelling and ordered by falling count, ties by spelling.
    fn ranked(self) -> Vec<(String, u32)> {
        let mut merged: BTreeMap<String, u32> = BTreeMap::new();
        for (word, count) in self.words {
            let total = merged.entry(word).or_default();
            *total = total.saturating_add(count);
        }
        let mut words: Vec<(String, u32)> = merged.into_iter().collect();
        words.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        words
    }
}
