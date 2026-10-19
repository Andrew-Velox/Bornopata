use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// One word-table entry: where the word sits in the arena and its unigram count. The index is the word id. 12 bytes, no padding, stored as is.
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct WordEntry {
    /// Byte offset in `BigramModel::words`.
    pub text_start: u32,

    /// Unigram count.
    pub count: u32,

    /// Byte length.
    pub text_len: u16,

    /// Alignment padding, all zero.
    pub reserved: u16,
}
