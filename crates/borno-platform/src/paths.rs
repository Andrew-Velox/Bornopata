//! Where Bornopata keeps its files: the XDG directories on Linux, `%APPDATA%` and `%LOCALAPPDATA%` on Windows.

use std::path::PathBuf;

/// The dictionary file name inside the user data directory.
pub const DICTIONARY_FILE: &str = "words.bnd";

/// The English gloss file next to the dictionary.
pub const GLOSS_FILE: &str = "gloss-en.bnd";

/// The language model of word pairs next to the dictionary.
pub const LANGUAGE_MODEL_FILE: &str = "lm.bnd";

/// The user's own phrases next to the dictionary.
pub const PHRASES_FILE: &str = "phrases.tsv";

/// Where the user left the candidate window after dragging it.
pub const WINDOW_POSITION_FILE: &str = "window-position";

/// A file the settings program drops to ask the Server to forget what it learned.
pub const FORGET_MARKER: &str = "forget-learned";

#[cfg(not(windows))]
fn xdg(variable: &str, fallback: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(fallback)))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("bornopata")
}

/// `%APPDATA%\Bornopata` and `%LOCALAPPDATA%\Bornopata`: settings and learned data roam with the user, logs do not.
#[cfg(windows)]
fn app_dir(variable: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Bornopata")
}

/// Learned data, the dictionary and the optional language model: `$XDG_DATA_HOME/bornopata`, on Windows
/// `%APPDATA%\Bornopata`.
pub fn user_dir() -> PathBuf {
    #[cfg(windows)]
    return app_dir("APPDATA");
    #[cfg(not(windows))]
    xdg("XDG_DATA_HOME", ".local/share")
}

/// `$XDG_CONFIG_HOME/bornopata/config.toml`, on Windows `%APPDATA%\Bornopata\config.toml`.
pub fn config_path() -> PathBuf {
    #[cfg(windows)]
    return app_dir("APPDATA").join("config.toml");
    #[cfg(not(windows))]
    xdg("XDG_CONFIG_HOME", ".config").join("config.toml")
}

/// Rolling Server logs: `$XDG_STATE_HOME/bornopata/logs`, on Windows `%LOCALAPPDATA%\Bornopata\logs`.
pub fn log_dir() -> PathBuf {
    #[cfg(windows)]
    return app_dir("LOCALAPPDATA").join("logs");
    #[cfg(not(windows))]
    xdg("XDG_STATE_HOME", ".local/state").join("logs")
}

/// The directories of system-wide data, where a package puts the dictionary: `bornopata` inside each of
/// `$XDG_DATA_DIRS` (default `/usr/local/share:/usr/share`); on Windows the `data` folder next to the program, where
/// the installer puts it.
fn system_dirs() -> Vec<PathBuf> {
    #[cfg(windows)]
    return std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("data")))
        .into_iter()
        .collect();
    #[cfg(not(windows))]
    {
        let dirs = std::env::var_os("XDG_DATA_DIRS")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
        std::env::split_paths(&dirs)
            .filter(|path| path.is_absolute())
            .map(|path| path.join("bornopata"))
            .collect()
    }
}

/// The first `name` that exists in `directories`, in order.
fn find_in(directories: &[PathBuf], name: &str) -> Option<PathBuf> {
    directories
        .iter()
        .map(|directory| directory.join(name))
        .find(|path| path.is_file())
}

/// A data file: the override variable if set, else the user directory, then the system directories. `None` when there
/// is no such file.
fn data_file(variable: &str, name: &str) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(variable).map(PathBuf::from) {
        return path.is_file().then_some(path);
    }
    let mut directories = vec![user_dir()];
    directories.extend(system_dirs());
    find_in(&directories, name)
}

/// The dictionary to use: `$BORNOPATA_DICT`, else `words.bnd` in the user directory or a system data directory. `None`
/// when there is no such file, in which case only the built-in scaffold word list answers.
pub fn dictionary_path() -> Option<PathBuf> {
    data_file("BORNOPATA_DICT", DICTIONARY_FILE)
}

/// The gloss file to use: `$BORNOPATA_GLOSS`, else `gloss-en.bnd` in the user directory or a system data directory.
/// `None` when there is no such file, in which case candidates have no gloss.
pub fn gloss_path() -> Option<PathBuf> {
    data_file("BORNOPATA_GLOSS", GLOSS_FILE)
}

/// The language model to use: `$BORNOPATA_LM`, else `lm.bnd` in the user directory or a system data directory. `None`
/// when there is no such file, in which case only what the user typed before ranks and suggests words.
pub fn language_model_path() -> Option<PathBuf> {
    data_file("BORNOPATA_LM", LANGUAGE_MODEL_FILE)
}

/// The phrase file the user edits: `phrases.tsv` in the user directory. It may not exist yet.
pub fn phrases_path() -> PathBuf {
    user_dir().join(PHRASES_FILE)
}

/// The marker file that asks the Server to forget everything it learned; the Server deletes it once it has.
pub fn forget_marker_path() -> PathBuf {
    user_dir().join(FORGET_MARKER)
}

/// The file that remembers where the user dropped the candidate window. It may not exist.
pub fn window_position_path() -> PathBuf {
    // Screens differ from one machine to the next, so on Windows this does not roam with the settings.
    #[cfg(windows)]
    return app_dir("LOCALAPPDATA").join(WINDOW_POSITION_FILE);
    #[cfg(not(windows))]
    user_dir().join(WINDOW_POSITION_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_directory_that_has_the_file_wins() {
        let base = std::env::temp_dir().join(format!("borno-paths-{}", std::process::id()));
        let (user, system) = (base.join("user"), base.join("system"));
        std::fs::create_dir_all(&user).unwrap();
        std::fs::create_dir_all(&system).unwrap();
        std::fs::write(system.join("words.bnd"), "system").unwrap();
        let both = [user.clone(), system.clone()];
        assert_eq!(find_in(&both, "words.bnd"), Some(system.join("words.bnd")));
        std::fs::write(user.join("words.bnd"), "user").unwrap();
        assert_eq!(find_in(&both, "words.bnd"), Some(user.join("words.bnd")));
        assert_eq!(find_in(&both, "missing.bnd"), None);
        std::fs::remove_dir_all(&base).unwrap();
    }
}
