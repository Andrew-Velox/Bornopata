//! Opening and validating the file.

use std::path::Path;

use borno_format::{Container, Kind};
use fst::{Map, Streamer};

use super::{Dictionary, Section};
use crate::error::DictionaryError;
use crate::layout::{
    ENTRIES_TAG, Group, KEYS_TAG, MEMBERS_TAG, PREFIX_WORDS_TAG, PREFIXES_TAG, WORDS_TAG,
};

impl Dictionary {
    /// Maps `path` and checks that every offset and id lies inside its section, so lookups cannot index out of
    /// bounds even on a damaged file.
    pub fn open(path: &Path) -> Result<Self, DictionaryError> {
        let container = Container::open(path, Kind::Dictionary)?;
        let keys = Map::new(Section(container.table(KEYS_TAG)?))?;
        let prefixes = Map::new(Section(container.table(PREFIXES_TAG)?))?;
        let dictionary = Self {
            keys,
            prefixes,
            members: container.table(MEMBERS_TAG)?,
            words: container.text(WORDS_TAG)?,
            entries: container.table(ENTRIES_TAG)?,
            prefix_words: container.table(PREFIX_WORDS_TAG)?,
            metadata: container.metadata().clone(),
        };
        dictionary.validate()?;
        tracing::debug!(words = dictionary.len(), name = %dictionary.metadata.name, "dictionary mapped");
        Ok(dictionary)
    }

    fn validate(&self) -> Result<(), DictionaryError> {
        let count = self.entries.len();
        for entry in self.entries.iter() {
            let start = entry.text_start as usize;
            let end = start + usize::from(entry.text_len);
            if self.words.get(start..end).is_none() {
                return Err(DictionaryError::Corrupt(
                    "word entry points outside the text",
                ));
            }
        }
        if self.members.iter().any(|&id| id as usize >= count)
            || self.prefix_words.iter().any(|&id| id as usize >= count)
        {
            return Err(DictionaryError::Corrupt("word id out of range"));
        }
        let ranges = [
            (&self.keys, self.members.len()),
            (&self.prefixes, self.prefix_words.len()),
        ];
        for (map, length) in ranges {
            let mut stream = map.stream();
            while let Some((_, value)) = stream.next() {
                let group = Group::from_value(value);
                if group.start + group.count > length {
                    return Err(DictionaryError::Corrupt("group points outside its table"));
                }
            }
        }
        Ok(())
    }
}
