//! Errors of the build tool.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{path}:{line}: expected `word<TAB>count`")]
    BadCountLine { path: PathBuf, line: usize },

    #[error("no corpus: give --corpus")]
    NoCorpus,

    #[error("no word pair is left after filtering; lower --min-count or check the input")]
    Empty,

    #[error("{0}")]
    Model(#[from] borno_lm::LmError),
}
