//! The learner: word counts, per-input choices and the personal bigram, with time decay and TSV persistence.

mod learner_impl;
mod persistence;

#[cfg(test)]
mod tests;

use std::cell::Cell;
use std::path::PathBuf;

use foldhash::{HashMap, HashSet};

use crate::decay::{DEFAULT_HALF_LIFE_DAYS, Decay, today};
use crate::user_ngram::UserNgram;

/// What the user committed, kept in memory and written to a directory of TSV files by `flush`.
///
/// Counts and choices cool with time (see [`Decay`]); the bigram does not decay but is capped.
#[derive(Debug)]
pub struct FrequencyLearner {
    /// Word to how often it was committed.
    counts: HashMap<String, Decay>,

    /// Typed input to (text to how often it was committed for that input).
    choices: HashMap<String, HashMap<String, Decay>>,

    ngram: UserNgram,

    /// Next-word suggestions the user rejected, as (previous word or `<s>`, word); they stay hidden whatever the
    /// language model says.
    hidden: HashSet<(String, String)>,

    half_life_days: f64,

    /// Today's unix day. Read paths are `&self` and must not query the clock each time; write paths refresh it.
    today: Cell<u64>,

    /// Changes since the last save.
    dirty: bool,

    /// Where `flush` writes; `None` keeps everything in memory.
    directory: Option<PathBuf>,
}

impl Default for FrequencyLearner {
    fn default() -> Self {
        Self {
            counts: HashMap::default(),
            choices: HashMap::default(),
            ngram: UserNgram::default(),
            hidden: HashSet::default(),
            half_life_days: DEFAULT_HALF_LIFE_DAYS,
            today: Cell::new(today()),
            dirty: false,
            directory: None,
        }
    }
}

impl FrequencyLearner {
    /// A learner that remembers within this process only.
    pub fn in_memory() -> Self {
        Self::default()
    }

    /// Forgets everything: words, choices, the personal bigram and the hidden suggestions. The files are emptied by the next `save`.
    pub fn clear(&mut self) {
        self.counts.clear();
        self.choices.clear();
        self.ngram = UserNgram::default();
        self.hidden.clear();
        self.dirty = true;
    }

    /// Overrides the decay half-life in days; 0 turns decay off.
    pub fn set_half_life_days(&mut self, days: f64) {
        self.half_life_days = days;
    }

    /// Whether there are changes not yet written.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Number of distinct committed words.
    pub fn word_count(&self) -> usize {
        self.counts.len()
    }

    fn effective(&self, entry: &Decay) -> u32 {
        entry.effective(self.today.get(), self.half_life_days)
    }
}
