//! Word-level bigram language model implementing Core's `LanguageModel` trait.
//! Loads unigram and bigram counts from TSV (`from_paths`) or maps a `.bnd` file directly (`from_path`, near-zero startup cost).
//!
//! The TSV files come from corpus statistics:
//!
//! ```text
//! lm-unigram.tsv    word\tcount        (`<s>` marks a sentence start; its count is the number of sentences)
//! lm-bigram.tsv     previous\tword\tcount
//! ```
//!
//! Probabilities are interpolated: P(w|v) = λ·c(v,w)/c(v) + (1-λ)·c(w)/N. Words the model lacks return `None`.

mod bigram_model;
mod error;
mod successor;
mod word_entry;

pub use bigram_model::{BigramModel, SENTENCE_START};
pub use error::LmError;
