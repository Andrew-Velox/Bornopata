//! The seam between the engine and wherever the phrases are kept.

/// Implemented by the shell's phrase store (a file the user edits). Core only asks.
pub trait PhraseSource: Send {
    /// The texts the user gave to exactly `code`, in the order they listed them. Empty when there are none.
    fn lookup(&self, code: &str) -> Vec<String>;
}
