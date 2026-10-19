//! What every session shares: the word source, the learner and the language model.
//!
//! Each Fcitx5 input context gets its own [`Engine`] (its own word being typed), but what was learned
//! has to be one table, so the engines talk to these shared handles.

mod learner;
mod model;
mod words;

use std::sync::{Arc, Mutex};

use borno_core::Engine;
use borno_learning::FrequencyLearner;
use borno_lm::BigramModel;
use borno_transliteration::WordSource;

use learner::SharedLearner;
use model::SharedModel;
use words::SharedWords;

/// Candidates the panel can show; the engine never returns more, so the highlight stays on screen.
const PANEL_CANDIDATES: usize = 9;

#[derive(Clone)]
pub struct Shared {
    words: Arc<dyn WordSource + Send + Sync>,
    learner: Option<Arc<Mutex<FrequencyLearner>>>,
    model: Option<Arc<BigramModel>>,
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
        }
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

    pub fn learning(&self) -> bool {
        self.learner.is_some()
    }
}
