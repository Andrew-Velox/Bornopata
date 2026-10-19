//! The self-drawn candidate window on Wayland.
//!
//! [`placement`] decides where the popup goes, [`compositor`] asks the compositor for the geometry that needs,
//! and [`buffer`] converts what `borno-render` draws for `wl_shm`.

pub mod buffer;
pub mod compositor;
mod geometry;
pub mod placement;

pub use geometry::Rect;
