//! Finding the theme the config names.

use borno_render::Theme;

use crate::command::PanelConfig;
use crate::error::PanelError;

/// The built-in theme `config.theme` names, or else the theme directory at that path.
pub fn load_theme(config: &PanelConfig) -> Result<Theme, PanelError> {
    if let Some(theme) = Theme::builtin(&config.theme, config.dark) {
        return Ok(theme);
    }
    Theme::from_dir(std::path::Path::new(&config.theme), config.dark)
        .map_err(|error| PanelError::Render(format!("theme {}: {error}", config.theme)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_themes_load_by_id_and_unknown_ones_are_an_error() {
        let config = PanelConfig {
            theme: "sakura".to_owned(),
            ..PanelConfig::default()
        };
        assert_eq!(load_theme(&config).unwrap().id(), "sakura");
        let missing = PanelConfig {
            theme: "/nonexistent/theme".to_owned(),
            ..PanelConfig::default()
        };
        assert!(matches!(load_theme(&missing), Err(PanelError::Render(_))));
    }
}
