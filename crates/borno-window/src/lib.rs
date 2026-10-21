//! What every self-drawn candidate window shares, whatever window system it sits on: how it is configured, which
//! theme the config names, and where it goes relative to the text cursor. Putting pixels on the screen is the
//! platform's business (`borno-panel` on Wayland, the Windows Server on Windows).

mod config;
mod error;
mod geometry;
pub mod placement;
mod theme;

pub use config::PanelConfig;
pub use error::WindowError;
pub use geometry::Rect;
pub use theme::load_theme;
