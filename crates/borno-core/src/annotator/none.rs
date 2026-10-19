//! The annotator of an engine without glosses.

use super::{Annotation, Annotator};

/// Annotates nothing; what an engine has until a real annotator is set.
pub struct NoAnnotator;

impl Annotator for NoAnnotator {
    fn annotate(&self, _word: &str) -> Option<Annotation> {
        None
    }
}
