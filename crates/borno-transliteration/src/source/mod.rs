//! The seam between Core and a Bangla word list.

mod fixed;
mod upodesh;

pub use fixed::FixedWordSource;
pub use upodesh::UpodeshWordSource;

/// Word suggestions for a Roman (phonetic) input, so the word list behind Core can be replaced.
/// See docs/design/bangla-reuse-plan.md for why the bundled list is only a scaffold.
pub trait WordSource {
    /// Candidate Bangla words for `roman`, best first. Empty when nothing matches.
    fn suggest(&self, roman: &str) -> Vec<String>;
}
