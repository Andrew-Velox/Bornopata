//! Where Bornopata keeps its files, following the XDG directories.

use std::path::PathBuf;

/// The dictionary file name inside the user data directory.
pub const DICTIONARY_FILE: &str = "words.bnd";

/// The English gloss file next to the dictionary.
pub const GLOSS_FILE: &str = "gloss-en.bnd";

/// The user's own phrases next to the dictionary.
pub const PHRASES_FILE: &str = "phrases.tsv";

/// A file the settings program drops to ask the Server to forget what it learned.
pub const FORGET_MARKER: &str = "forget-learned";

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

/// The gloss file to use: `$BORNOPATA_GLOSS`, else `gloss-en.bnd` in the user directory. `None` when there is no such
/// file, in which case candidates have no gloss.
pub fn gloss_path() -> Option<PathBuf> {
    let path = std::env::var_os("BORNOPATA_GLOSS")
        .map(PathBuf::from)
        .unwrap_or_else(|| user_dir().join(GLOSS_FILE));
    path.is_file().then_some(path)
}

/// The phrase file the user edits: `phrases.tsv` in the user directory. It may not exist yet.
pub fn phrases_path() -> PathBuf {
    user_dir().join(PHRASES_FILE)
}

/// The marker file that asks the Server to forget everything it learned; the Server deletes it once it has.
pub fn forget_marker_path() -> PathBuf {
    user_dir().join(FORGET_MARKER)
}
