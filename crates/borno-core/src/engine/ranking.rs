//! Candidate scoring. The constants are first guesses; tune them with `borno-cli` once there is frequency data.

use crate::candidate::CandidateSource;

/// The direct conversion that no dictionary word backs: shown, but behind real words.
const UNCONFIRMED_PRIOR: f64 = 1.0;
const DICTIONARY_PRIOR: f64 = 1.5;

/// The direct conversion that is also a dictionary word.
const CONFIRMED_PRIOR: f64 = 1.7;

/// Added to the direct conversion when the typed letters include a capital and the conversion is a real word: in Avro
/// a capital is part of the spelling (`T` is ট, `t` is ত), so that word must not lose to what the context makes
/// likelier. A capital that makes no word (`ShikkhA`) is only a habit, and the dictionary word still wins.
const EXPLICIT_CASE_BONUS: f64 = 1.0;

/// A word that only starts with what was typed. These are listed after every spelling of the input.
const COMPLETION_PRIOR: f64 = 1.0;
const CHOICE_WEIGHT: f64 = 4.0;

/// Kept small on purpose: how often a word was committed for some other input must not outvote the letters typed now.
const WORD_WEIGHT: f64 = 0.2;
const PAIR_WEIGHT: f64 = 0.5;
const LM_WEIGHT: f64 = 0.2;

/// Weight of the corpus frequency, on a log10 scale: a word seen a million times adds 3.6, one seen ten times 0.6.
const FREQUENCY_WEIGHT: f64 = 0.6;

/// Cost of each letter by which a dictionary spelling differs from what the Avro rules made of the input.
const DISTANCE_PENALTY: f64 = 0.1;

/// Edits beyond this cost nothing more.
const MAX_DISTANCE: usize = 5;

/// `log P` values below this add nothing: the model barely knows the pair.
const LM_FLOOR: f64 = -10.0;

/// A corrected spelling beats any dictionary word, but the user's own repeated choice for the same input beats it
/// again after a few picks (`CHOICE_WEIGHT` times the log of the count).
const AUTOCORRECT_PRIOR: f64 = 6.0;

/// A phrase of the user outranks everything else; later lines of the file go after earlier ones.
const PHRASE_PRIOR: f64 = 1000.0;
const PHRASE_STEP: f64 = 10.0;

/// Everything known about one candidate text when it is scored.
#[derive(Debug, Default)]
pub(super) struct Evidence {
    /// Position of this text among the user's phrases for the input, when it is one.
    pub phrase: Option<usize>,

    /// The autocorrect table gives this text for the input.
    pub autocorrect: bool,

    /// The direct conversion produced this text.
    pub direct: bool,

    /// The direct conversion of an input that has a capital letter in it.
    pub explicit_case: bool,

    /// The word source suggested this text as a spelling of the input.
    pub dictionary: bool,

    /// The word source suggested this text only because it starts with the input.
    pub completion: bool,

    /// Corpus frequency from the word source; 0 when it did not say.
    pub frequency: u32,

    /// Letters by which the text differs from the direct conversion.
    pub distance: usize,

    pub choice_weight: u32,
    pub word_weight: u32,
    pub pair_count: u32,
    pub log_prob: Option<f64>,
}

impl Evidence {
    /// Learned wins over direct, direct over dictionary, dictionary over completion.
    pub fn source(&self) -> CandidateSource {
        if self.phrase.is_some() {
            CandidateSource::Phrase
        } else if self.choice_weight > 0 {
            CandidateSource::Learned
        } else if self.autocorrect {
            CandidateSource::Autocorrect
        } else if self.direct {
            CandidateSource::Direct
        } else if self.dictionary {
            CandidateSource::Dictionary
        } else {
            CandidateSource::Completion
        }
    }

    /// Completion-only candidates go after all the others whatever their score, so the first candidate, which a
    /// Space commits, is always something the user typed.
    pub fn is_completion_only(&self) -> bool {
        self.source() == CandidateSource::Completion
    }

    pub fn score(&self) -> f64 {
        let prior = match (self.direct, self.dictionary, self.completion) {
            (true, true, _) => CONFIRMED_PRIOR,
            (false, true, _) => DICTIONARY_PRIOR,
            (true, false, _) => UNCONFIRMED_PRIOR,
            (false, false, true) => COMPLETION_PRIOR,
            (false, false, false) => 0.0,
        };
        let language_model = self
            .log_prob
            .map_or(0.0, |log_prob| LM_WEIGHT * (log_prob - LM_FLOOR).max(0.0));
        let spelling_gap = if self.dictionary && !self.direct {
            DISTANCE_PENALTY * self.distance.min(MAX_DISTANCE) as f64
        } else {
            0.0
        };
        let explicit_case = if self.direct && self.dictionary && self.explicit_case {
            EXPLICIT_CASE_BONUS
        } else {
            0.0
        };
        let phrase = self
            .phrase
            .map_or(0.0, |position| PHRASE_PRIOR - PHRASE_STEP * position as f64);
        let autocorrect = if self.autocorrect {
            AUTOCORRECT_PRIOR
        } else {
            0.0
        };
        phrase + autocorrect + prior + explicit_case - spelling_gap
            + FREQUENCY_WEIGHT * f64::from(self.frequency).log10_1p()
            + CHOICE_WEIGHT * f64::from(self.choice_weight).ln_1p()
            + WORD_WEIGHT * f64::from(self.word_weight).ln_1p()
            + PAIR_WEIGHT * f64::from(self.pair_count).ln_1p()
            + language_model
    }
}

trait Log10OnePlus {
    fn log10_1p(self) -> f64;
}

impl Log10OnePlus for f64 {
    fn log10_1p(self) -> f64 {
        (1.0 + self).log10()
    }
}

/// Levenshtein distance in characters.
pub(super) fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, &ca) in a.iter().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (row[j] + 1)
                .min(above + 1)
                .min(diagonal + usize::from(ca != cb));
            diagonal = above;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::edit_distance;

    #[test]
    fn counts_letter_edits() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("কথা", "কথা"), 0);
        assert_eq!(edit_distance("কথা", "কোথা"), 1);
        assert_eq!(edit_distance("কথা", ""), 3);
        assert_eq!(edit_distance("abc", "xbz"), 2);
    }
}
