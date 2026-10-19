//! Short glosses for Bangla words, shown beside a candidate (`n. rice`).
//!
//! One gloss per word in one auxiliary language: a part of speech and a text of at most two meanings. The data is a
//! `.bnd` file of kind glossary (`gloss-en.bnd` next to the dictionary), mapped and read without copying; a lookup is
//! a hash probe, so it can run while the candidate window is built without delaying candidate generation.
//! [`Glossary`] implements Core's `Annotator`; [`GlossaryBuilder`] writes the file.

mod builder;
mod error;
mod gloss;
mod glossary;
mod record;

#[cfg(test)]
mod tests;

pub use builder::GlossaryBuilder;
pub use error::GlossaryError;
pub use gloss::Gloss;
pub use glossary::Glossary;

/// Longest gloss text kept, in bytes: a row of the candidate window is one short line.
pub const MAX_TEXT_BYTES: usize = 120;

/// Longest part-of-speech label kept, in bytes.
pub const MAX_POS_BYTES: usize = 8;
