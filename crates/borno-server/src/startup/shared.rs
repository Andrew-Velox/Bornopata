//! The engine parts every session shares: the word list, the learned data, the language model, the glosses and the
//! user's phrases.

use std::path::Path;
use std::sync::Arc;

use borno_gloss::Glossary;
use borno_learning::FrequencyLearner;
use borno_lm::BigramModel;
use borno_platform::{Config, paths};

use crate::error::ServerError;
use crate::shared::{FilePhrases, Shared};

/// What a new `phrases.tsv` starts with.
const PHRASE_TEMPLATE: &str = include_str!("phrases-template.tsv");

/// Opens everything the config asks for. A data file that is missing only takes its feature away, and is logged.
pub fn build_shared(config: &Config) -> Result<Shared, ServerError> {
    let user_dir = paths::user_dir();
    let learner = if config.general.learning {
        Some(FrequencyLearner::open(&user_dir)?)
    } else {
        None
    };
    let model = load_model()?;
    let dictionary = paths::dictionary_path();
    match &dictionary {
        Some(path) => tracing::info!(path = %path.display(), "using the dictionary"),
        None => tracing::warn!("no dictionary file, only the scaffold word list answers"),
    }
    let words = borno_dictionary::word_source(dictionary.as_deref())?;
    let glossary = load_glossary(config);
    write_phrase_template(&user_dir);
    Ok(Shared::new(Arc::from(words), learner, model)
        .with_glossary(glossary)
        .with_shortcuts(config.general.shortcuts)
        .with_emoji(config.general.emoji)
        .with_autocorrect(config.general.autocorrect)
        .with_phrases(FilePhrases::open(paths::phrases_path())))
}

/// The English glosses beside the candidates, when they are turned on and the file is there. A file that cannot be read
/// is logged and the candidates simply have no gloss.
fn load_glossary(config: &Config) -> Option<Glossary> {
    if !config.general.gloss {
        tracing::info!("glosses are turned off in the config");
        return None;
    }
    let Some(path) = paths::gloss_path() else {
        tracing::info!("no gloss file, candidates have no gloss");
        return None;
    };
    match Glossary::open(&path) {
        Ok(glossary) => {
            tracing::info!(path = %path.display(), entries = glossary.len(), "using the glosses");
            Some(glossary)
        }
        Err(error) => {
            tracing::warn!(%error, path = %path.display(), "the gloss file cannot be used");
            None
        }
    }
}

/// The phrase file with its instructions, so the user finds it where the Server looks. Never overwrites.
fn write_phrase_template(user_dir: &Path) {
    let path = user_dir.join(paths::PHRASES_FILE);
    if path.exists() {
        return;
    }
    if let Err(error) =
        std::fs::create_dir_all(user_dir).and_then(|()| std::fs::write(&path, PHRASE_TEMPLATE))
    {
        tracing::warn!(%error, path = %path.display(), "cannot write the phrase template");
    }
}

fn load_model() -> Result<Option<BigramModel>, ServerError> {
    let Some(path) = paths::language_model_path() else {
        return Ok(None);
    };
    let model = BigramModel::from_path(&path)?;
    tracing::info!(
        path = %path.display(),
        words = model.word_count(),
        pairs = model.bigram_count(),
        "using the language model"
    );
    Ok(Some(model))
}
