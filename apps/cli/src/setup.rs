//! Builds the engine from the arguments.

use borno_core::{Engine, LanguageModel};
use borno_learning::FrequencyLearner;
use borno_lm::BigramModel;

use crate::args::Args;
use crate::error::CliError;

/// The word source is the dictionary when there is one, with the upodesh scaffold behind it
/// (docs/design/bangla-reuse-plan.md).
pub fn build_engine(args: &Args) -> Result<Engine, CliError> {
    let learner = match &args.user_dir {
        Some(directory) => FrequencyLearner::open(directory)?,
        None => FrequencyLearner::in_memory(),
    };
    let dictionary = if args.no_dict {
        None
    } else {
        args.dict
            .clone()
            .or_else(borno_platform::paths::dictionary_path)
    };
    let words = borno_dictionary::word_source(dictionary.as_deref())?;
    let mut engine = Engine::new(words).with_learner(Box::new(learner));
    if let Some(model) = language_model(args)? {
        engine = engine.with_language_model(model);
    }
    Ok(engine)
}

fn language_model(args: &Args) -> Result<Option<Box<dyn LanguageModel>>, CliError> {
    match (&args.lm, &args.lm_unigram, &args.lm_bigram) {
        (Some(path), None, None) => Ok(Some(Box::new(BigramModel::from_path(path)?))),
        (None, Some(unigram), Some(bigram)) => {
            Ok(Some(Box::new(BigramModel::from_paths(unigram, bigram)?)))
        }
        (None, None, None) => Ok(None),
        _ => Err(CliError::IncompleteLanguageModel),
    }
}
