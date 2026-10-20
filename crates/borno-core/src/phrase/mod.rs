//! The user's own phrases: a short code typed in Roman letters that expands to any text.

mod book;
mod none;
mod problem;
mod source;

pub use book::{Parsed, PhraseBook};
pub use none::NoPhrases;
pub use problem::PhraseProblem;
pub use source::PhraseSource;
