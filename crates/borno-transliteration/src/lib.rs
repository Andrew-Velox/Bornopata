//! Roman to Bangla transliteration and word suggestions. Direct conversion by the Avro rules and the word source seam.

pub mod source;
mod transliterator;

pub use source::{
    FallbackWordSource, FixedWordSource, UpodeshWordSource, WordSource, WordSuggestion,
};
pub use transliterator::Transliterator;
