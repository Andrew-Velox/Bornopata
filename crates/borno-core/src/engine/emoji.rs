//! Emoji next to the words they stand for.

use std::collections::HashSet;

use super::Engine;
use crate::candidate::{Candidate, CandidateSource};

/// Only the best few words are looked up, and only until one of them has emoji: a spelling further down the list is
/// a guess, and its emoji would be noise next to the real word's.
const WORDS_LOOKED_AT: usize = 3;

/// Emoji offered after one word.
const PER_WORD: usize = 2;

/// Emoji in one list, so the words keep most of the panel.
const MAX_EMOJI: usize = 3;

/// An English word is looked up only from this length on; two letters match too many Roman spellings of Bangla words.
const MIN_ENGLISH_LEN: usize = 3;

impl Engine {
    /// Puts emoji right after the words that name them. The first word also gets those of the English word typed in
    /// Roman letters (`heart`, which Avro turns into nonsense).
    pub(super) fn insert_emoji(&self, input: &str, ranked: &mut Vec<Candidate>) {
        let english =
            input.len() >= MIN_ENGLISH_LEN && input.bytes().all(|b| b.is_ascii_lowercase());
        let mut known: HashSet<String> = ranked.iter().map(|c| c.text.clone()).collect();
        let mut added = 0;
        let mut searching = true;
        let mut out = Vec::with_capacity(ranked.len() + MAX_EMOJI);
        for (index, candidate) in std::mem::take(ranked).into_iter().enumerate() {
            let mut found = Vec::new();
            let lookable = searching
                && index < WORDS_LOOKED_AT
                && !matches!(
                    candidate.source,
                    CandidateSource::Completion | CandidateSource::Emoji
                );
            if lookable && added < MAX_EMOJI {
                found = self.emoji.lookup(&candidate.text);
                found.truncate(PER_WORD);
                searching = found.is_empty();
                if index == 0 && english {
                    let mut by_english = self.emoji.lookup(input);
                    by_english.truncate(PER_WORD);
                    found.extend(by_english);
                }
            }
            let (score, mut at) = (candidate.score, out.len() + 1);
            out.push(candidate);
            for emoji in found {
                if added == MAX_EMOJI || !known.insert(emoji.clone()) {
                    continue;
                }
                out.insert(
                    at,
                    Candidate {
                        text: emoji,
                        source: CandidateSource::Emoji,
                        score,
                    },
                );
                at += 1;
                added += 1;
            }
        }
        *ranked = out;
    }
}
