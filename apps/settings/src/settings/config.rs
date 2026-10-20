//! Writing the config file: change one setting in place, then read the file again.

use borno_platform::{Config, Setting};

use super::Settings;

impl Settings {
    pub(super) fn write(&mut self, section: &str, key: &str, setting: Setting) {
        match Config::set(&self.path, section, key, setting) {
            Ok(()) => {
                self.problem = None;
                self.reload();
            }
            Err(error) => {
                tracing::error!(%error, section, key, "could not write the config");
                self.problem = Some(error.to_string());
            }
        }
    }

    /// The text of the program follows the `language` setting.
    pub(super) fn apply_language(&self) {
        crate::i18n::use_bangla(self.config.general.language.uses_bangla());
    }

    pub(super) fn reload(&mut self) {
        match Config::load(&self.path) {
            Ok(config) => {
                self.config = config;
                self.apply_language();
            }
            Err(error) => {
                tracing::error!(%error, "could not read the config again");
                self.problem = Some(error.to_string());
            }
        }
    }
}
