//! The seam between Core and a Bangla word list.

mod fallback;
mod fixed;
mod suggestion;
mod upodesh;

pub use fallback::FallbackWordSource;
pub use fixed::FixedWordSource;
pub use suggestion::WordSuggestion;
pub use upodesh::UpodeshWordSource;

/// Word suggestions for a Roman (phonetic) input, so the word list behind Core can be replaced.
/// See docs/design/bangla-reuse-plan.md for why the bundled list is only a scaffold.
pub trait WordSource {
    /// Candidate Bangla words for `roman`. Empty when nothing matches. A source that ranks its words by frequency
    /// should return them best first; the scaffold list cannot, so for now the order is only stable, not meaningful.
    fn suggest(&self, roman: &str) -> Vec<String>;

    /// Up to `limit` words for the typed `roman`, with frequencies, and, for a source with an index, words that
    /// merely start with what was typed. The default wraps [`Self::suggest`] without frequencies or completions.
    fn lookup(&self, roman: &str, limit: usize) -> Vec<WordSuggestion> {
        self.suggest(roman)
            .into_iter()
            .take(limit)
            .map(WordSuggestion::plain)
            .collect()
    }
}
