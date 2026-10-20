//! Server log verbosity.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
}

impl LogLevel {
    /// The `tracing` filter directive for this level.
    pub fn directive(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            // Our own crates only: the text shaping library is far too chatty at debug level.
            Self::Debug => "info,borno_linux_server=debug,borno_panel=debug",
        }
    }
}
