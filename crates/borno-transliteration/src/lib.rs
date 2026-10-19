//! Roman to Bangla transliteration and word suggestions. Only the word source seam exists so far.

pub mod source;

pub use source::{FixedWordSource, UpodeshWordSource, WordSource};
