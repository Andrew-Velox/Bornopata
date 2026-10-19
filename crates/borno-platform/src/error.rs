//! Errors of reading and writing the config file.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write config {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid config {path}: {source}")]
    Parse {
        path: PathBuf,

        /// Boxed: `toml::de::Error` is over a hundred bytes and would push every error type that wraps this one
        /// past clippy's size threshold.
        #[source]
        source: Box<toml::de::Error>,
    },
}
