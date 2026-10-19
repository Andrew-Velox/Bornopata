//! The fixed-width record of the entry table.

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// One glossary entry: where its word, part of speech and text are in the text arena. 20 bytes, no padding.
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct EntryRecord {
    pub word_start: u32,

    pub pos_start: u32,

    pub text_start: u32,

    pub word_len: u16,

    pub pos_len: u16,

    pub text_len: u16,

    /// Keeps the record a multiple of four bytes; always zero.
    pub reserved: u16,
}

/// Sections of the file: the text arena, the entry table and the hash index over the words.
pub const TEXT_TAG: [u8; 4] = *b"TEXT";
pub const ENTRIES_TAG: [u8; 4] = *b"ENTR";
pub const HASH_TAG: [u8; 4] = *b"HASH";
