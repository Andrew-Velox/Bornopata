//! The Bangla word list: cleaning corpus text into words, phonetic keys, and (next) the dictionary file.

mod key;
mod normalize;

pub use key::key;
pub use normalize::{normalize, tokens};
