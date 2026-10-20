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

    /// Show a short English gloss with its part of speech beside each Bangla candidate (`n. rice`). It needs the
    /// glossary file `gloss-en.bnd` in the data directory; without it nothing is shown.
    pub gloss: bool,

    /// Typing `tarikh`, `somoy` or `bar` also offers today's date, the time or the weekday, in Bangla.
    pub shortcuts: bool,

    pub log_level: LogLevel,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            learning: true,
            bangla_digits: true,
            gloss: true,
            shortcuts: true,
            log_level: LogLevel::default(),
        }
    }
}
