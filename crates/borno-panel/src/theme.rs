//! Finding the theme the config names, with the panel's error.

use borno_render::Theme;
use borno_window::PanelConfig;

use crate::error::PanelError;

/// The built-in theme `config.theme` names, or else the theme directory at that path.
pub fn load_theme(config: &PanelConfig) -> Result<Theme, PanelError> {
    borno_window::load_theme(config).map_err(|error| PanelError::Render(error.to_string()))
}
