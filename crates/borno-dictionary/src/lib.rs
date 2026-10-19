//! The Bangla word list: cleaning corpus text into words, phonetic keys, and the dictionary file.

mod builder;
mod dictionary;
mod error;
mod key;
mod layout;
mod normalize;

pub use builder::DictionaryBuilder;
pub use dictionary::{Dictionary, Match};
pub use error::DictionaryError;
pub use key::key;
pub use normalize::{normalize, tokens};
