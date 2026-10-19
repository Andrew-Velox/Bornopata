//! Looking words up by key: the words with exactly this key, then the best words whose key continues it.

use std::collections::BinaryHeap;

use fst::automaton::{Automaton, Str};
use fst::{IntoStreamer, Streamer};

use super::{Dictionary, Match};
use crate::layout::{Group, PRECOMPUTED_PREFIX_CHARS};

/// Keys looked at for a prefix with no precomputed list; bounds the time of a pathological prefix.
const MAX_KEYS_SCANNED: usize = 20_000;

impl Dictionary {
    /// Up to `limit` words for the phonetic `key` (see [`crate::key`]): first every word whose key is exactly
    /// `key`, most frequent first, then the most frequent words whose key starts with `key` and goes on.
    pub fn lookup(&self, key: &str, limit: usize) -> Vec<Match<'_>> {
        if key.is_empty() || limit == 0 {
            return Vec::new();
        }
        let exact: &[u32] = match self.keys.get(key.as_bytes()).map(Group::from_value) {
            Some(group) => &self.members[group.start..group.start + group.count],
            None => &[],
        };
        let mut found: Vec<Match<'_>> = exact
            .iter()
            .take(limit)
            .map(|&id| self.matched(id, true))
            .collect();
        if found.len() >= limit {
            return found;
        }
        // The exact words come back among the completions too; ask for enough to drop them and still fill the list.
        for id in self.completions(key, limit + exact.len().min(limit)) {
            if found.len() == limit {
                break;
            }
            if !exact.contains(&id) {
                found.push(self.matched(id, false));
            }
        }
        found
    }

    /// The `limit` most frequent word ids whose key starts with `prefix` (including the exact key, callers drop it).
    pub(crate) fn completions(&self, prefix: &str, limit: usize) -> Vec<u32> {
        if prefix.chars().count() <= PRECOMPUTED_PREFIX_CHARS {
            return self.precomputed(prefix, limit);
        }
        self.scan(prefix, limit)
    }

    fn precomputed(&self, prefix: &str, limit: usize) -> Vec<u32> {
        let Some(value) = self.prefixes.get(prefix.as_bytes()) else {
            return Vec::new();
        };
        let group = Group::from_value(value);
        self.prefix_words[group.start..group.start + group.count]
            .iter()
            .copied()
            .take(limit)
            .collect()
    }

    /// Streams the keys under `prefix` and keeps the best `limit` ids. Ids are ranks, so the best are the smallest.
    pub(crate) fn scan(&self, prefix: &str, limit: usize) -> Vec<u32> {
        let mut best: BinaryHeap<u32> = BinaryHeap::with_capacity(limit + 1);
        let mut stream = self
            .keys
            .search(Str::new(prefix).starts_with())
            .into_stream();
        let mut scanned = 0;
        while let Some((_, value)) = stream.next() {
            scanned += 1;
            let group = Group::from_value(value);
            for &id in self.members[group.start..group.start + group.count]
                .iter()
                .take(limit)
            {
                if best.len() < limit {
                    best.push(id);
                } else if best.peek().is_some_and(|&worst| id < worst) {
                    best.pop();
                    best.push(id);
                } else {
                    // The group is ascending: nothing later in it can beat the current worst.
                    break;
                }
            }
            if scanned >= MAX_KEYS_SCANNED {
                break;
            }
        }
        best.into_sorted_vec()
    }
}
