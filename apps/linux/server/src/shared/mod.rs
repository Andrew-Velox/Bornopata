//! What every session shares: the word source, the learner and the language model.
//!
//! Each Fcitx5 input context gets its own [`Engine`] (its own word being typed), but what was learned
//! has to be one table, so the engines talk to these shared handles.

mod clock;
mod emoji;
mod learner;
mod model;
mod phrases;
mod words;

use std::sync::{Arc, Mutex};

use borno_core::Engine;
use borno_emoji::EmojiSet;
use borno_gloss::Glossary;
use borno_learning::FrequencyLearner;
use borno_lm::BigramModel;
use borno_transliteration::WordSource;

use clock::SystemClock;
use emoji::SharedEmoji;
use learner::SharedLearner;
use model::SharedModel;
use phrases::SharedPhrases;
use words::SharedWords;

pub use phrases::FilePhrases;

/// Candidates the panel can show; the engine never returns more, so the highlight stays on screen.
const PANEL_CANDIDATES: usize = 9;

#[derive(Clone)]
pub struct Shared {
    words: Arc<dyn WordSource + Send + Sync>,
    learner: Option<Arc<Mutex<FrequencyLearner>>>,
    model: Option<Arc<BigramModel>>,
    glossary: Option<Arc<Glossary>>,
    phrases: Option<Arc<FilePhrases>>,
    shortcuts: bool,
    emoji: Option<Arc<EmojiSet>>,
}

impl Shared {
    /// `learner` is `None` when learning is turned off in the config.
    pub fn new(
        words: Arc<dyn WordSource + Send + Sync>,
        learner: Option<FrequencyLearner>,
        model: Option<BigramModel>,
    ) -> Self {
        Self {
            words,
            learner: learner.map(|learner| Arc::new(Mutex::new(learner))),
            model: model.map(Arc::new),
            glossary: None,
            phrases: None,
            shortcuts: false,
            emoji: None,
        }
    }

    /// Shows the gloss of each candidate from `glossary`, which every engine shares.
    pub fn with_glossary(mut self, glossary: Option<Glossary>) -> Self {
        self.glossary = glossary.map(Arc::new);
        self
    }

    /// Gives every engine the user's phrases from `phrases`, which follows the file as it is edited.
    pub fn with_phrases(mut self, phrases: FilePhrases) -> Self {
        self.phrases = Some(Arc::new(phrases));
        self
    }

    /// Turns on the date and time shortcuts (`tarikh`, `somoy`, `bar`).
    pub fn with_shortcuts(mut self, enabled: bool) -> Self {
        self.shortcuts = enabled;
        self
    }

    /// Offers emoji next to the words they stand for; the table is built into the program.
    pub fn with_emoji(mut self, enabled: bool) -> Self {
        self.emoji = enabled.then(|| Arc::new(EmojiSet::bundled()));
        self
    }

    /// A new engine for one input context.
    pub fn engine(&self) -> Engine {
        let mut engine = Engine::new(Box::new(SharedWords(self.words.clone())))
            .with_candidate_limit(PANEL_CANDIDATES);
        if let Some(learner) = &self.learner {
            engine = engine.with_learner(Box::new(SharedLearner(learner.clone())));
        }
        if let Some(model) = &self.model {
            engine = engine.with_language_model(Box::new(SharedModel(model.clone())));
        }
        if let Some(emoji) = &self.emoji {
            engine = engine.with_emoji(Box::new(SharedEmoji(emoji.clone())));
        }
        if self.shortcuts {
            engine = engine.with_clock(Box::new(SystemClock));
        }
        if let Some(phrases) = &self.phrases {
            engine = engine.with_phrases(Box::new(SharedPhrases(phrases.clone())));
        }
        if let Some(glossary) = &self.glossary {
            engine = engine.with_annotator(Box::new(glossary.clone()));
        }
        engine
    }

    /// Writes learned data to disk. Returns whether anything was written.
    pub fn flush(&self) -> std::io::Result<()> {
        match &self.learner {
            Some(learner) => learner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .save(),
            None => Ok(()),
        }
    }

    /// Forgets everything learned, in memory and on disk.
    pub fn forget_learned(&self) -> std::io::Result<()> {
        match &self.learner {
            Some(learner) => {
                let mut learner = learner
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                learner.clear();
                learner.save()
            }
            None => Ok(()),
        }
    }

    pub fn learning(&self) -> bool {
        self.learner.is_some()
    }
}
