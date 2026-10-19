//! Errors of the build steps.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("cannot write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("{path} line {line}: {reason}")]
    Line {
        path: PathBuf,
        line: usize,
        reason: String,
    },

    #[error("glossary: {0}")]
    Glossary(#[from] borno_gloss::GlossaryError),

    #[error("nothing to write: no entries")]
    Empty,
}
