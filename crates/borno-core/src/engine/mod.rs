//! The engine facade: composition buffer, candidates, commit and next-word suggestions.

mod annotations;
mod candidates;
mod emoji;
mod next_words;
mod query;
mod ranking;
mod timings;
mod typed;
mod typing;

#[cfg(test)]
mod tests;

pub use query::Query;
pub use timings::Timings;
pub use typed::Typed;

use borno_transliteration::{Transliterator, WordSource};

use crate::annotator::{Annotator, NoAnnotator};
use crate::autocorrect::{AutocorrectSource, NoAutocorrect};
use crate::candidate::Candidate;
use crate::composition::Composition;
use crate::emoji::{EmojiSource, NoEmoji};
use crate::learner::{Learner, NoLearner};
use crate::lm::{LanguageModel, NoLanguageModel};
use crate::phrase::{NoPhrases, PhraseSource};
use crate::shortcut::Clock;

/// The only entry point a shell needs. Single-threaded state: one engine per input context.
pub struct Engine {
    transliterator: Transliterator,
    words: Box<dyn WordSource + Send>,
    learner: Box<dyn Learner>,
    language_model: Box<dyn LanguageModel>,
    annotator: Box<dyn Annotator>,
    phrases: Box<dyn PhraseSource>,
    autocorrect: Box<dyn AutocorrectSource>,
    emoji: Box<dyn EmojiSource>,
    clock: Option<Box<dyn Clock>>,

    /// The word being typed.
    composition: Composition,

    /// Candidates for `composition`, best first; recomputed on every change of the composition.
    candidates: Vec<Candidate>,

    /// Index into `candidates` of the highlighted one.
    highlighted: usize,

    /// The last committed word of the current sentence, the context for ranking and next-word suggestions.
    previous: Option<String>,

    /// Timings of the last candidate generation.
    timings: Timings,

    /// Whether commits are remembered. Off for private fields.
    learning: bool,

    /// Most candidates a query returns; the rest are dropped after ranking.
    candidate_limit: usize,
}

impl Engine {
    /// An engine with no learning and no language model.
    pub fn new(words: Box<dyn WordSource + Send>) -> Self {
        Self {
            transliterator: Transliterator::new(),
            words,
            learner: Box::new(NoLearner),
            language_model: Box::new(NoLanguageModel),
            annotator: Box::new(NoAnnotator),
            phrases: Box::new(NoPhrases),
            autocorrect: Box::new(NoAutocorrect),
            emoji: Box::new(NoEmoji),
            clock: None,
            composition: Composition::default(),
            candidates: Vec::new(),
            highlighted: 0,
            previous: None,
            timings: Timings::default(),
            learning: true,
            candidate_limit: usize::MAX,
        }
    }

    pub fn with_learner(mut self, learner: Box<dyn Learner>) -> Self {
        self.learner = learner;
        self
    }

    pub fn with_language_model(mut self, language_model: Box<dyn LanguageModel>) -> Self {
        self.language_model = language_model;
        self
    }

    /// Sets the source of the annotation shown beside each candidate; see [`Engine::annotations`].
    pub fn with_annotator(mut self, annotator: Box<dyn Annotator>) -> Self {
        self.annotator = annotator;
        self
    }

    /// Adds the user's own phrases: a text given to exactly the typed code comes first in the list.
    pub fn with_phrases(mut self, phrases: Box<dyn PhraseSource>) -> Self {
        self.phrases = phrases;
        self
    }

    /// Puts the Bangla spelling of an English word typed as written (`facebook`) ahead of what the Avro rules make of it.
    pub fn with_autocorrect(mut self, autocorrect: Box<dyn AutocorrectSource>) -> Self {
        self.autocorrect = autocorrect;
        self
    }

    /// Offers emoji from `emoji` next to the words they stand for.
    pub fn with_emoji(mut self, emoji: Box<dyn EmojiSource>) -> Self {
        self.emoji = emoji;
        self
    }

    /// Turns on the date and time shortcuts (`tarikh`, `somoy`, `bar`), which read the time from `clock`.
    pub fn with_clock(mut self, clock: Box<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Turns remembering on or off. While off, commits still update the sentence context but teach the learner
    /// nothing; ranking keeps using what was learned before.
    pub fn set_learning(&mut self, enabled: bool) {
        self.learning = enabled;
    }

    /// Caps the candidate list. The shell's panel shows a fixed number, so the highlight must stay inside it.
    pub fn with_candidate_limit(mut self, limit: usize) -> Self {
        self.candidate_limit = limit.max(1);
        self
    }

    pub fn composition(&self) -> &Composition {
        &self.composition
    }

    /// Candidates for the current composition, best first. Empty when nothing is being typed.
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    pub fn highlighted(&self) -> usize {
        self.highlighted
    }

    /// The last committed word of the current sentence.
    pub fn previous_word(&self) -> Option<&str> {
        self.previous.as_deref()
    }

    /// Timings of the last candidate generation.
    pub fn timings(&self) -> Timings {
        self.timings
    }

    /// Writes learned data to disk. Called by the shell when it is idle, never from the typing path.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.learner.flush()
    }
}
