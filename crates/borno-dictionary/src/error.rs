//! Errors of reading and building the dictionary file.

use borno_format::FormatError;

#[derive(Debug, thiserror::Error)]
pub enum DictionaryError {
    #[error("dictionary file: {0}")]
    Format(#[from] FormatError),

    #[error("dictionary index: {0}")]
    Index(#[from] fst::Error),

    #[error("dictionary file is corrupt: {0}")]
    Corrupt(&'static str),
}
