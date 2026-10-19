//! Where a candidate came from, for the shell to mark.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    /// The user chose this for this input before.
    Learned,

    /// The direct Avro conversion.
    Direct,

    /// A word from the word list.
    Dictionary,

    /// A word the typed letters are only the beginning of.
    Completion,

    /// A next-word suggestion.
    Next,
}
