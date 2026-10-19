//! Errors of reading and building a glossary file.

use borno_format::FormatError;

#[derive(Debug, thiserror::Error)]
pub enum GlossaryError {
    #[error("glossary file: {0}")]
    Format(#[from] FormatError),

    #[error("glossary file is corrupt: {0}")]
    Corrupt(&'static str),

    #[error("line {line}: {reason}")]
    Line { line: usize, reason: String },

    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
