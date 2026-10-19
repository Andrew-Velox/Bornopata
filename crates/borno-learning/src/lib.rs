//! On-device learning for Bornopata: what the user committed, remembered between sessions.
//!
//! [`FrequencyLearner`] implements Core's `Learner` and stores TSV files in one directory:
//! word counts, per-input choices and the personal bigram. Counts cool with time ([`Decay`]).

mod decay;
mod error;
mod frequency_learner;
mod user_ngram;

pub use decay::{COLD_DAYS, DEFAULT_HALF_LIFE_DAYS, Decay};
pub use error::LearningError;
pub use frequency_learner::FrequencyLearner;
pub use user_ngram::UserNgram;
