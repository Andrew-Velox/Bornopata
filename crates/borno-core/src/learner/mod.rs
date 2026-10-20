//! The seam to user-side learning.

mod none;

pub use none::NoLearner;

/// What the user chose, remembered between sessions. Implemented by `borno-learning`.
///
/// Every method is an in-memory lookup or update: nothing here may block candidate generation or write to disk
/// ([`Self::flush`] is the only call that does, and the shell decides when).
pub trait Learner: Send {
    /// The user committed `text` for the typed `input`, right after `previous` (`None` at the start of a sentence).
    fn record(&mut self, input: &str, text: &str, previous: Option<&str>);

    /// How often `text` was committed, whatever the input.
    fn weight(&self, _text: &str) -> u32 {
        0
    }

    /// How often `text` was committed for exactly this `input`.
    fn choice_weight(&self, _input: &str, _text: &str) -> u32 {
        0
    }

    /// Texts committed for exactly this `input`, most chosen first.
    fn choices(&self, _input: &str) -> Vec<(String, u32)> {
        Vec::new()
    }

    /// How often `text` followed `previous`.
    fn pair_count(&self, _previous: &str, _text: &str) -> u32 {
        0
    }

    /// Words that followed `previous` (`None` for a sentence start), most frequent first, at most `limit`.
    fn followers(&self, _previous: Option<&str>, _limit: usize) -> Vec<(String, u32)> {
        Vec::new()
    }

    /// Takes back what was learned about `text`: the choice of it for `input`, and, when no other input still chooses
    /// it (or `input` is empty), its word count and its place among the followers of other words. Returns whether
    /// anything was removed.
    fn forget(&mut self, _input: &str, _text: &str) -> bool {
        false
    }

    /// Stops `text` being offered as a next-word suggestion after `previous` (`None` for a sentence start), even when
    /// it comes from the language model, which cannot be edited. Returns whether it was newly hidden.
    fn hide_suggestion(&mut self, _previous: Option<&str>, _text: &str) -> bool {
        false
    }

    /// Whether `text` was hidden as a suggestion after `previous`.
    fn is_suggestion_hidden(&self, _previous: Option<&str>, _text: &str) -> bool {
        false
    }

    /// Writes pending changes to disk.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
