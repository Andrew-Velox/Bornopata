//! Corrections for words the Avro rules cannot spell, mostly English words typed as they are written.

mod none;
mod source;

pub use none::NoAutocorrect;
pub use source::AutocorrectSource;
