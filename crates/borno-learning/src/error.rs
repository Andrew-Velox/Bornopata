//! Errors of the learning data files.

#[derive(Debug, thiserror::Error)]
pub enum LearningError {
    /// Reading or writing a learning data file failed (permissions, bad disk). Malformed lines are skipped, not errors.
    #[error("failed to read or write user learning file: {0}")]
    Io(#[from] std::io::Error),
}
