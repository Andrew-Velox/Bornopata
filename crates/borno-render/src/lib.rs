//! The self-drawn candidate window: one frame (typed letters, candidate rows, hint) drawn by a theme into a bitmap.
//! The shell only puts the bitmap on screen.
//!
//! Drawing is done by `bamboo-slip`; this crate owns what is specific to the candidate window: the frame data, the
//! theme format, building the scene tree from a theme template, and the click regions. All sizes in a [`Theme`]
//! are in points; the [`Renderer`] multiplies them by the scale factor.
//!
//! Design: `docs/design/candidate-window.md`.

mod frame;
mod icon;
mod layout;
pub mod preview;
mod renderer;
pub mod sample;
mod theme;

pub use bamboo_slip::{Color, FontLibrary, FontSpec, Pixmap, RenderError, UiFont};
pub use frame::{Frame, Mode, Preedit, PreeditSegment, PreeditStyle, Row, Source, Tone};
pub use layout::Layout;
pub use renderer::{HitRegion, HitTarget, Rendered, Renderer};
pub use theme::{TextSizes, Theme, ThemeError, ThemeLibrary};
