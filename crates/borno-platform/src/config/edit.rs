//! Changing one setting in the config file in place, so the user's comments and layout survive.

use std::path::Path;

use toml_edit::{DocumentMut, Item, value};

use super::{Config, TEMPLATE};
use crate::error::ConfigError;

/// What a setting can be set to.
#[derive(Debug, Clone, PartialEq)]
pub enum Setting {
    Bool(bool),
    Text(String),
}

impl Config {
    /// Sets `[section] key = setting` in the file at `path` and writes it back. A missing file starts from the commented
    /// template. The rest of the file is untouched, comments included; a file that is not valid TOML is an error and is
    /// left as it is.
    pub fn set(path: &Path, section: &str, key: &str, setting: Setting) -> Result<(), ConfigError> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => TEMPLATE.to_owned(),
            Err(source) => {
                return Err(ConfigError::Read {
                    path: path.to_owned(),
                    source,
                });
            }
        };
        let mut document: DocumentMut = text.parse().map_err(|source| ConfigError::Edit {
            path: path.to_owned(),
            source: Box::new(source),
        })?;
        let table = document
            .entry(section)
            .or_insert_with(|| Item::Table(toml_edit::Table::new()));
        let Some(table) = table.as_table_like_mut() else {
            return Ok(());
        };
        let new = match setting {
            Setting::Bool(flag) => value(flag),
            Setting::Text(text) => value(text),
        };
        // Keep the comment above an existing key: only the value is replaced.
        match table.get_mut(key) {
            Some(Item::Value(old)) => {
                let decor = old.decor().clone();
                let mut replaced = new.into_value().expect("a plain value");
                *replaced.decor_mut() = decor;
                *old = replaced;
            }
            _ => {
                table.insert(key, new);
            }
        }
        write_atomic(path, &document.to_string())
    }
}

/// Written next to the file and renamed over it, so a reader (the Server watching it) never sees half a file.
fn write_atomic(path: &Path, text: &str) -> Result<(), ConfigError> {
    let write = |source| ConfigError::Write {
        path: path.to_owned(),
        source,
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(write)?;
    }
    let temporary = path.with_extension("toml.tmp");
    std::fs::write(&temporary, text).map_err(write)?;
    std::fs::rename(&temporary, path).map_err(write)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let directory =
            std::env::temp_dir().join(format!("borno-edit-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        directory.join("config.toml")
    }

    #[test]
    fn a_value_changes_and_the_comments_stay() {
        let path = scratch("comments");
        std::fs::write(
            &path,
            "# my notes\n[general]\n# keep it private\nlearning = true # inline\nemoji = true\n",
        )
        .unwrap();
        Config::set(&path, "general", "learning", Setting::Bool(false)).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("# my notes"));
        assert!(text.contains("# keep it private"));
        assert!(text.contains("learning = false # inline"), "{text}");
        assert!(text.contains("emoji = true"));
        assert!(!Config::load(&path).unwrap().general.learning);
    }

    #[test]
    fn a_missing_key_or_section_is_added() {
        let path = scratch("add");
        std::fs::write(&path, "[general]\nlearning = true\n").unwrap();
        Config::set(&path, "panel", "theme", Setting::Text("sakura".into())).unwrap();
        Config::set(&path, "general", "emoji", Setting::Bool(false)).unwrap();
        let config = Config::load(&path).unwrap();
        assert_eq!(config.panel.theme, "sakura");
        assert!(!config.general.emoji);
        assert!(config.general.learning);
    }

    #[test]
    fn a_missing_file_starts_from_the_template() {
        let path = scratch("missing");
        Config::set(&path, "general", "gloss", Setting::Bool(false)).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("# Bornopata configuration"));
        assert!(!Config::load(&path).unwrap().general.gloss);
    }

    #[test]
    fn a_broken_file_is_an_error_and_stays_as_it_is() {
        let path = scratch("broken");
        std::fs::write(&path, "[general\nlearning = true\n").unwrap();
        assert!(Config::set(&path, "general", "learning", Setting::Bool(false)).is_err());
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "[general\nlearning = true\n"
        );
    }
}
