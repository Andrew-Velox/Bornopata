//! The `[general]` section.

use serde::{Deserialize, Serialize};

use super::LogLevel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Remember what you choose, so it comes first next time. Learned data stays on this machine.
    pub learning: bool,

    /// Type digits as Bangla digits (০-৯), as Avro does. When off, digits pass through unchanged.
    pub bangla_digits: bool,

    pub log_level: LogLevel,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            learning: true,
            bangla_digits: true,
            log_level: LogLevel::default(),
        }
    }
}
