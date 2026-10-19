//! Config reading: defaults, overrides, errors and the template.

use super::*;

fn scratch(name: &str) -> std::path::PathBuf {
    let directory =
        std::env::temp_dir().join(format!("borno-config-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory.join("config.toml")
}

#[test]
fn the_template_parses_to_the_defaults() {
    let parsed: Config = toml::from_str(Config::template()).unwrap();
    assert_eq!(parsed, Config::default());
    assert!(parsed.general.learning);
    assert!(parsed.general.bangla_digits);
    assert_eq!(parsed.shortcut.toggle.spelling(), "ctrl+alt+space");
}

#[test]
fn a_missing_file_gives_the_defaults() {
    let path = scratch("missing");
    assert_eq!(Config::load(&path).unwrap(), Config::default());
}

#[test]
fn values_override_and_unknown_keys_are_ignored() {
    let path = scratch("override");
    std::fs::write(
        &path,
        "[general]\nlearning = false\nlog_level = \"debug\"\nfuture_key = 1\n[shortcut]\ntoggle = \"ctrl+shift+b\"\n[future]\nx = 2\n",
    )
    .unwrap();
    let config = Config::load(&path).unwrap();
    assert!(!config.general.learning);
    assert!(config.general.bangla_digits);
    assert_eq!(config.general.log_level, LogLevel::Debug);
    assert_eq!(config.shortcut.toggle.spelling(), "ctrl+shift+b");
}

#[test]
fn a_bad_shortcut_is_an_error_naming_the_file() {
    let path = scratch("bad");
    std::fs::write(&path, "[shortcut]\ntoggle = \"space\"\n").unwrap();
    let message = Config::load(&path).unwrap_err().to_string();
    assert!(message.contains("config.toml"), "{message}");
    assert!(message.contains("modifier"), "{message}");
}

#[test]
fn the_template_is_written_once() {
    let path = scratch("template");
    assert!(Config::write_template_if_missing(&path).unwrap());
    std::fs::write(&path, "[general]\nlearning = false\n").unwrap();
    assert!(!Config::write_template_if_missing(&path).unwrap());
    assert!(!Config::load(&path).unwrap().general.learning);
}
