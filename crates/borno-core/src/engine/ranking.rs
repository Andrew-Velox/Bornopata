//! Candidate scoring. The constants are first guesses; tune them with `borno-cli` once there is frequency data.

use crate::candidate::CandidateSource;

/// The direct conversion that no dictionary word backs: shown, but behind real words.
const UNCONFIRMED_PRIOR: f64 = 1.0;
const DICTIONARY_PRIOR: f64 = 1.5;

/// The direct conversion that is also a dictionary word.
const CONFIRMED_PRIOR: f64 = 2.0;
const CHOICE_WEIGHT: f64 = 3.0;
/// Kept small on purpose: how often a word was committed for some other input must not outvote the letters typed now.
const WORD_WEIGHT: f64 = 0.2;
const PAIR_WEIGHT: f64 = 0.5;
const LM_WEIGHT: f64 = 0.2;

/// `log P` values below this add nothing: the model barely knows the pair.
const LM_FLOOR: f64 = -10.0;

/// Everything known about one candidate text when it is scored.
#[derive(Debug, Default)]
pub(super) struct Evidence {
    /// The direct conversion produced this text.
    pub direct: bool,

    /// The word source suggested this text. Its order carries no frequency yet, so the position is not used.
    pub dictionary: bool,

    pub choice_weight: u32,
    pub word_weight: u32,
    pub pair_count: u32,
    pub log_prob: Option<f64>,
}

impl Evidence {
    /// Learned wins over direct, direct over dictionary.
    pub fn source(&self) -> CandidateSource {
        if self.choice_weight > 0 {
            CandidateSource::Learned
        } else if self.direct {
            CandidateSource::Direct
        } else {
            CandidateSource::Dictionary
        }
    }

    pub fn score(&self) -> f64 {
        let prior = match (self.direct, self.dictionary) {
            (true, true) => CONFIRMED_PRIOR,
            (false, true) => DICTIONARY_PRIOR,
            (true, false) => UNCONFIRMED_PRIOR,
            (false, false) => 0.0,
        };
        let language_model = self
            .log_prob
            .map_or(0.0, |log_prob| LM_WEIGHT * (log_prob - LM_FLOOR).max(0.0));
        prior
            + CHOICE_WEIGHT * f64::from(self.choice_weight).ln_1p()
            + WORD_WEIGHT * f64::from(self.word_weight).ln_1p()
            + PAIR_WEIGHT * f64::from(self.pair_count).ln_1p()
            + language_model
    }
}
