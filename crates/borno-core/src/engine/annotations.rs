//! Annotations of the current candidates.

use super::Engine;
use crate::annotator::Annotation;
use crate::candidate::CandidateSource;

impl Engine {
    /// The annotation of any `word`, for words that are not candidates (next-word suggestions).
    pub fn annotate(&self, word: &str) -> Option<Annotation> {
        self.annotator.annotate(word)
    }

    /// The annotation of each current candidate, in order. Called after [`Engine::candidates`] when the window is
    /// built, so candidate generation never waits for it; ranking does not use it.
    pub fn annotations(&self) -> Vec<Option<Annotation>> {
        self.candidates
            .iter()
            .map(|candidate| match candidate.source {
                CandidateSource::Emoji => self.emoji.name(&candidate.text).map(|name| Annotation {
                    pos: String::new(),
                    text: name,
                }),
                _ => self.annotator.annotate(&candidate.text),
            })
            .collect()
    }
}
