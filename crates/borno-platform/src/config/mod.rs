//! The config file (`config.toml`): read at Server start, written as a commented template when missing.

mod general;
mod key_combo;
mod log_level;
mod modifiers;
mod shortcut;

#[cfg(test)]
mod tests;

use std::path::Path;

use serde::{Deserialize, Serialize};

pub use general::GeneralConfig;
pub use key_combo::{ComboKey, KeyCombo};
pub use log_level::LogLevel;
pub use modifiers::Modifiers;
pub use shortcut::ShortcutConfig;

use crate::error::ConfigError;

/// What gets written when the file does not exist yet. Every value shown is the default.
const TEMPLATE: &str = r#"# Bornopata configuration. Restart the Server after editing.

[general]
# Remember what you choose so it comes first next time. Learned data stays on this machine.
learning = true

# Type digits as Bangla digits (০-৯). Set to false to keep ASCII digits.
bangla_digits = true

# error, warn, info or debug
log_level = "info"

[shortcut]
# Switch between Bangla and English typing. Modifiers (ctrl, alt, shift, super) plus space, a letter or a digit.
toggle = "ctrl+alt+space"
"#;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub shortcut: ShortcutConfig,
}

impl Config {
    /// Reads `path`. A missing file gives the defaults; a malformed one is an error so a typo is not silently ignored.
    /// Unknown keys are ignored, so a newer file still loads.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(source) => {
                return Err(ConfigError::Read {
                    path: path.to_owned(),
                    source,
                });
            }
        };
        toml::from_str(&text).map_err(|source| ConfigError::Parse {
            path: path.to_owned(),
            source: Box::new(source),
        })
    }

    /// Writes the commented template when `path` does not exist. Returns whether it wrote.
    pub fn write_template_if_missing(path: &Path) -> Result<bool, ConfigError> {
        if path.exists() {
            return Ok(false);
        }
        std::fs::write(path, TEMPLATE).map_err(|source| ConfigError::Write {
            path: path.to_owned(),
            source,
        })?;
        Ok(true)
    }

    /// The template text, for tests and documentation.
    pub fn template() -> &'static str {
        TEMPLATE
    }
}
