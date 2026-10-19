use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// One successor in the CSR: the next word's id and the (previous, next) count. 8 bytes, stored as is.
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Successor {
    /// Id of the next word.
    pub word: u32,

    /// Bigram count.
    pub count: u32,
}
