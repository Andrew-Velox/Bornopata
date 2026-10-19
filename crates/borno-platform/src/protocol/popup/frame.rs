//! One bitmap to show.

use serde::{Deserialize, Serialize};

/// A bitmap in a slot of a shared pool, premultiplied `ARGB8888` (`wl_shm` format 0), cropped to the card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PopupFrame {
    /// The shell's identifier of the input context the frame is for; a frame for a context that is not the one
    /// showing the popup is dropped (and released).
    pub context: String,

    /// The pool the bitmap lives in, from an earlier [`super::PopupServerMessage::Pool`].
    pub pool: u64,

    /// Names this slot use until it is released.
    pub buffer: u64,

    /// Byte offset of the bitmap in the pool.
    pub offset: u32,

    /// Size and row length in pixels and bytes.
    pub width: u32,

    pub height: u32,

    pub stride: u32,

    /// The scale the bitmap was drawn at, in 120ths.
    pub scale_120: u32,

    /// The size the popup has on screen, in logical pixels: the viewport destination.
    pub logical_width: u32,

    pub logical_height: u32,
}
