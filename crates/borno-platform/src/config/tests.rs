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
    assert!(parsed.general.gloss);
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

#[test]
fn the_panel_defaults_and_overrides() {
    let defaults = Config::default();
    assert_eq!(defaults.panel.mode, PanelMode::Auto);
    assert_eq!(defaults.panel.theme, "bornopata");
    assert!(!defaults.panel.dark);
    assert_eq!(defaults.panel.layout, PanelLayout::Vertical);

    let path = scratch("panel");
    std::fs::write(
        &path,
        "[panel]\nmode = \"fcitx\"\ntheme = \"sakura\"\ndark = true\nlayout = \"horizontal\"\n",
    )
    .unwrap();
    let config = Config::load(&path).unwrap();
    assert_eq!(config.panel.mode, PanelMode::Fcitx);
    assert_eq!(config.panel.theme, "sakura");
    assert!(config.panel.dark);
    assert_eq!(config.panel.layout, PanelLayout::Horizontal);

    std::fs::write(&path, "[panel]\nmode = \"popup\"\n").unwrap();
    assert_eq!(Config::load(&path).unwrap().panel.mode, PanelMode::Popup);

    std::fs::write(&path, "[panel]\nmode = \"sideways\"\n").unwrap();
    assert!(Config::load(&path).is_err());
}

#[test]
fn the_gloss_setting_defaults_on_and_can_be_turned_off() {
    assert!(Config::default().general.gloss);
    let path = scratch("gloss");
    std::fs::write(&path, "[general]\ngloss = false\n").unwrap();
    assert!(!Config::load(&path).unwrap().general.gloss);
}

#[test]
fn the_language_reads_as_auto_en_or_bn_and_defaults_to_auto() {
    assert_eq!(Config::default().general.language, Language::Auto);
    for (text, want) in [
        ("auto", Language::Auto),
        ("en", Language::English),
        ("bn", Language::Bangla),
    ] {
        let config: Config =
            toml::from_str(&format!("[general]\nlanguage = \"{text}\"\n")).unwrap();
        assert_eq!(config.general.language, want, "{text}");
    }
}
