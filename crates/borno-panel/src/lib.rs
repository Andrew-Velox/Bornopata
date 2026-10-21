//! The self-drawn candidate window on Wayland.
//!
//! [`placement`] decides where the popup goes, [`compositor`] asks the compositor for the geometry that needs,
//! and [`buffer`] converts what `borno-render` draws for `wl_shm`.

pub mod buffer;
mod command;
pub mod compositor;
mod error;
mod theme;
mod wayland;

pub use borno_window::{PanelConfig, Rect, placement};
pub use error::PanelError;
pub use theme::load_theme;
pub use wayland::Panel;
