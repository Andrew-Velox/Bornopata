//! The frame the shell draws for a session.

use borno_core::Annotation;
use borno_core::CandidateSource;
use borno_platform::protocol::{Frame, FrameCandidate, FrameGloss, Source};

use super::Session;

fn gloss(annotation: Option<Annotation>) -> Option<FrameGloss> {
    annotation.map(|annotation| FrameGloss {
        pos: annotation.pos,
        text: annotation.text,
    })
}

fn source(source: CandidateSource) -> Source {
    match source {
        // The user's own text is marked like a learned choice; the wire format has no separate kind for it.
        CandidateSource::Phrase | CandidateSource::Learned => Source::Learned,
        CandidateSource::Direct => Source::Direct,
        CandidateSource::Dictionary => Source::Dictionary,
        CandidateSource::Completion => Source::Completion,
    }
}

impl Session {
    /// What to show now: the word being typed with its candidates, or the next-word suggestions after a space,
    /// or nothing.
    pub fn frame(&self) -> Frame {
        if !self.engine.composition().is_empty() {
            // The candidates exist first; their glosses are looked up afterwards, from the mapped table.
            let annotations = self.engine.annotations();
            let candidates: Vec<FrameCandidate> = self
                .engine
                .candidates()
                .iter()
                .zip(annotations)
                .map(|(candidate, annotation)| FrameCandidate {
                    text: candidate.text.clone(),
                    source: source(candidate.source),
                    gloss: gloss(annotation),
                })
                .collect();
            let highlight = self
                .engine
                .highlighted()
                .min(candidates.len().saturating_sub(1));
            return Frame {
                roman: self.engine.composition().as_str().to_owned(),
                preedit: candidates
                    .get(highlight)
                    .map(|candidate| candidate.text.clone())
                    .unwrap_or_default(),
                candidates,
                highlight,
                suggesting: false,
                mode: self.mode,
                native: false,
            };
        }
        if !self.suggestions.is_empty() {
            return Frame {
                candidates: self
                    .suggestions
                    .iter()
                    .map(|text| FrameCandidate {
                        text: text.clone(),
                        source: Source::Next,
                        gloss: gloss(self.engine.annotate(text)),
                    })
                    .collect(),
                highlight: self.suggestion_highlight,
                suggesting: true,
                mode: self.mode,
                ..Frame::default()
            };
        }
        Frame {
            mode: self.mode,
            ..Frame::default()
        }
    }
}
