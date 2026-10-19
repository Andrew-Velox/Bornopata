//! The precomputed best words for the shortest key prefixes.

use std::collections::BTreeMap;

use fst::MapBuilder;

use crate::error::DictionaryError;
use crate::layout::{Group, PRECOMPUTED_PREFIX_CHARS, PRECOMPUTED_WORDS};

/// Returns the prefix FST bytes and the table of word ids it points into.
///
/// `groups` maps each key to its word ids in ascending order; walking the keys by their first word keeps the
/// lists short, and ids are already ranked, so the smallest ids of a prefix are its best words.
pub(super) fn build(
    groups: &BTreeMap<String, Vec<u32>>,
) -> Result<(Vec<u8>, Vec<u32>), DictionaryError> {
    let mut best: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    for (key, ids) in groups {
        let mut end = 0;
        for c in key.chars().take(PRECOMPUTED_PREFIX_CHARS) {
            end += c.len_utf8();
            let list = best.entry(key[..end].to_owned()).or_default();
            list.extend(ids.iter().take(PRECOMPUTED_WORDS));
        }
    }
    let mut index = MapBuilder::memory();
    let mut table: Vec<u32> = Vec::new();
    for (prefix, mut ids) in best {
        ids.sort_unstable();
        ids.dedup();
        ids.truncate(PRECOMPUTED_WORDS);
        let group = Group {
            start: table.len(),
            count: ids.len(),
        };
        table.extend_from_slice(&ids);
        index.insert(prefix.as_bytes(), group.to_value())?;
    }
    Ok((index.into_inner()?, table))
}
