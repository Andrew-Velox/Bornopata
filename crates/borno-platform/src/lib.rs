//! Parts shared by the Server and the shells that need no system API: the config file and the wire protocol.

pub mod config;
mod error;
pub mod paths;
pub mod protocol;

pub use config::{
    Config, GeneralConfig, KeyCombo, Language, LogLevel, Modifiers, PanelConfig, PanelLayout,
    PanelMode, Setting, ShortcutConfig,
};
pub use error::ConfigError;
