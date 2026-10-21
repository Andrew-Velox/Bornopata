//! The seam between the engine and the table of corrections.

/// Implemented by `borno-autocorrect`. Core only asks.
pub trait AutocorrectSource: Send {
    /// The Bangla spellings of `input` read as a written word (`facebook`, `Facebooke`), best first. Empty when the
    /// input is not one.
    fn lookup(&self, input: &str) -> Vec<String>;
}
