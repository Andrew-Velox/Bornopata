//! The seam to the optional annotation shown beside a candidate: a short gloss in one auxiliary language.

mod annotation;
mod none;

use std::sync::Arc;

pub use annotation::Annotation;
pub use none::NoAnnotator;

/// Looks up the annotation of a word. Implemented by `borno-gloss`. A lookup must be cheap (a hash probe in a mapped
/// table): it runs after the candidates exist, while the candidate window is built, and never delays them.
pub trait Annotator: Send {
    /// `None` when there is no annotation for `word`.
    fn annotate(&self, word: &str) -> Option<Annotation>;
}

/// One table shared by every input context's engine.
impl<T: Annotator + Sync + ?Sized> Annotator for Arc<T> {
    fn annotate(&self, word: &str) -> Option<Annotation> {
        (**self).annotate(word)
    }
}
