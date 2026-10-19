//! How the dictionary is laid out in its `.bnd` file.
//!
//! Words are numbered by falling frequency, so a smaller id is a more frequent word and every list of ids kept in
//! ascending order is already ranked.
//!
//! Sections: `KEYS` an FST from phonetic key to a group of words that share it; `MEMB` the word ids of all groups,
//! one group after the other; `WORD` the words' text; `WENT` one entry per word; `PFXS` and `PFXL` an FST from
//! each key prefix of one to three characters to the most frequent words under it.

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub(crate) const KEYS_TAG: [u8; 4] = *b"KEYS";
pub(crate) const MEMBERS_TAG: [u8; 4] = *b"MEMB";
pub(crate) const WORDS_TAG: [u8; 4] = *b"WORD";
pub(crate) const ENTRIES_TAG: [u8; 4] = *b"WENT";
pub(crate) const PREFIXES_TAG: [u8; 4] = *b"PFXS";
pub(crate) const PREFIX_WORDS_TAG: [u8; 4] = *b"PFXL";

/// Prefixes up to this many characters have their best words precomputed.
pub(crate) const PRECOMPUTED_PREFIX_CHARS: usize = 3;

/// Words kept per precomputed prefix.
pub(crate) const PRECOMPUTED_WORDS: usize = 16;

/// Bits of an FST value that hold the number of ids; the rest is the start index.
const COUNT_BITS: u32 = 20;

/// One word: where its text sits in the arena and how often it was seen. 12 bytes, stored as is.
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C)]
pub(crate) struct WordEntry {
    pub text_start: u32,

    /// Occurrences in the corpus.
    pub count: u32,

    pub text_len: u16,

    /// Padding, zero.
    pub reserved: u16,
}

/// A run of ids in a member table: `start` is an index, `count` how many ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Group {
    pub start: usize,
    pub count: usize,
}

impl Group {
    /// Largest group that can be stored.
    pub const MAX_COUNT: usize = (1 << COUNT_BITS) - 1;

    pub fn to_value(self) -> u64 {
        ((self.start as u64) << COUNT_BITS) | self.count as u64
    }

    pub fn from_value(value: u64) -> Self {
        Self {
            start: (value >> COUNT_BITS) as usize,
            count: (value & Self::MAX_COUNT as u64) as usize,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_group_survives_the_value_encoding() {
        for group in [
            Group { start: 0, count: 0 },
            Group { start: 7, count: 3 },
            Group {
                start: 123_456_789,
                count: Group::MAX_COUNT,
            },
        ] {
            assert_eq!(Group::from_value(group.to_value()), group);
        }
    }
}
