//! Where Bornopata keeps its files, following the XDG directories.

use std::path::PathBuf;

/// The dictionary file name inside the user data directory.
pub const DICTIONARY_FILE: &str = "words.bnd";

fn xdg(variable: &str, fallback: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(fallback)))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("bornopata")
}

/// Learned data, the dictionary and the optional language model: `$XDG_DATA_HOME/bornopata`.
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

/// The dictionary to use: `$BORNOPATA_DICT`, else `words.bnd` in the user directory. `None` when there is no such
/// file, in which case only the built-in scaffold word list answers.
pub fn dictionary_path() -> Option<PathBuf> {
    let path = std::env::var_os("BORNOPATA_DICT")
        .map(PathBuf::from)
        .unwrap_or_else(|| user_dir().join(DICTIONARY_FILE));
    path.is_file().then_some(path)
}
