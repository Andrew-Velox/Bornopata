//! What can go wrong while building the table.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("cannot write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{0} is not a version such as 15.1")]
    Version(String),
}
