//! Where the Server keeps its files, following the XDG directories.

use std::path::PathBuf;

fn xdg(variable: &str, fallback: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(fallback)))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("bornopata")
}

/// Learned data and the optional language model: `$XDG_DATA_HOME/bornopata`.
pub fn user_dir() -> PathBuf {
    xdg("XDG_DATA_HOME", ".local/share")
}

/// `$XDG_CONFIG_HOME/bornopata/config.toml`.
pub fn config_path() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config").join("config.toml")
}

/// Rolling Server logs: `$XDG_STATE_HOME/bornopata/logs`.
pub fn log_dir() -> PathBuf {
    xdg("XDG_STATE_HOME", ".local/state").join("logs")
}
