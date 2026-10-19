use thiserror::Error;

#[derive(Debug, Error)]
pub enum LmError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("bad line {line} in {file}: {reason}")]
    Format {
        /// File name.
        file: &'static str,

        /// Line number, from 1.
        line: usize,

        /// What is wrong.
        reason: &'static str,
    },

    #[error("language model file: {0}")]
    Container(#[from] borno_format::FormatError),

    #[error("corrupt language model file: {0}")]
    Corrupt(&'static str),
}
