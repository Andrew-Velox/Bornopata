//! The config file: written from the template when it is missing, then read.

use std::path::PathBuf;

use borno_platform::{Config, paths};

use crate::error::ServerError;

/// The path of the config file and what it says. A missing file is created from the template first.
pub fn load_config() -> Result<(PathBuf, Config), ServerError> {
    let path = paths::config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Config::write_template_if_missing(&path)?;
    let config = Config::load(&path)?;
    Ok((path, config))
}
