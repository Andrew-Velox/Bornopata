//! Candidate generation: direct conversion, word source suggestions and learned choices, ranked.

use std::collections::HashMap;
use std::time::Instant;

use super::ranking::{Evidence, edit_distance};
use super::{Engine, Query, Timings};
use crate::candidate::{Candidate, CandidateSource};
use crate::shortcut;

/// Longer inputs get the direct conversion only: the word source is built for words, not paragraphs.
const MAX_SUGGEST_LEN: usize = 64;

/// Words asked of the word source for one input; the candidate limit trims the list afterwards.
const LOOKUP_LIMIT: usize = 32;

impl Engine {
    /// Candidates for `input` as if typed right after `previous`. Does not touch the composition or learning state.
    pub fn query_after(&self, previous: Option<&str>, input: &str) -> Query {
        let (candidates, timings) = self.generate(previous, input);
        Query {
            input: input.to_owned(),
            candidates,
            timings,
        }
    }

    /// Candidates for `input` at the start of a sentence.
    pub fn query(&self, input: &str) -> Query {
        self.query_after(None, input)
    }

    /// Recomputes `candidates` for the current composition.
    pub(super) fn refresh_candidates(&mut self) {
        let (candidates, timings) =
            self.generate(self.previous.as_deref(), self.composition.as_str());
        self.candidates = candidates;
        self.timings = timings;
        self.highlighted = 0;
    }

    fn generate(&self, previous: Option<&str>, input: &str) -> (Vec<Candidate>, Timings) {
        let mut timings = Timings::default();
        if input.is_empty() {
            return (Vec::new(), timings);
        }

        // Insertion order only fixes which source is seen first; equal scores tie-break on text for a stable order.
        let mut order: Vec<String> = Vec::new();
        let mut evidence: HashMap<String, Evidence> = HashMap::new();

        let started = Instant::now();
        let direct = self.transliterator.convert(input);
        timings.direct = started.elapsed();
        if !direct.is_empty() {
            order.push(direct.clone());
            evidence.entry(direct.clone()).or_default().direct = true;
        }

        let started = Instant::now();
        let suggested = if input.len() <= MAX_SUGGEST_LEN {
            self.words.lookup(input, LOOKUP_LIMIT)
        } else {
            Vec::new()
        };
        for suggestion in suggested {
            let entry = evidence.entry(suggestion.text.clone()).or_insert_with(|| {
                order.push(suggestion.text.clone());
                Evidence::default()
            });
            if suggestion.completion {
                entry.completion = true;
            } else {
                entry.dictionary = true;
            }
            entry.frequency = entry.frequency.max(suggestion.frequency);
        }
        timings.dictionary = started.elapsed();

        for (position, text) in self.phrases.lookup(input).into_iter().enumerate() {
            let entry = evidence.entry(text.clone()).or_insert_with(|| {
                order.push(text);
                Evidence::default()
            });
            entry.phrase.get_or_insert(position);
        }

        let started = Instant::now();
        for (text, _) in self.learner.choices(input) {
            evidence.entry(text.clone()).or_insert_with(|| {
                order.push(text);
                Evidence::default()
            });
        }
        let mut ranked: Vec<(bool, Candidate)> = order
            .into_iter()
            .map(|text| {
                let mut known = evidence.remove(&text).unwrap_or_default();
                known.choice_weight = self.learner.choice_weight(input, &text);
                known.word_weight = self.learner.weight(&text);
                known.pair_count =
                    previous.map_or(0, |previous| self.learner.pair_count(previous, &text));
                known.log_prob = self.language_model.log_prob(previous, &text);
                known.explicit_case = input.chars().any(|c| c.is_ascii_uppercase());
                if known.dictionary && !known.direct {
                    known.distance = edit_distance(&text, &direct);
                }
                let candidate = Candidate {
                    source: known.source(),
                    score: known.score(),
                    text,
                };
                (known.is_completion_only(), candidate)
            })
            .collect();
        ranked.sort_by(|(a_completion, a), (b_completion, b)| {
            a_completion
                .cmp(b_completion)
                .then_with(|| b.score.total_cmp(&a.score))
                .then_with(|| a.text.cmp(&b.text))
        });
        let mut ranked: Vec<Candidate> =
            ranked.into_iter().map(|(_, candidate)| candidate).collect();
        self.insert_shortcuts(input, &mut ranked);
        self.insert_emoji(input, &mut ranked);
        ranked.truncate(self.candidate_limit);
        timings.ranking = started.elapsed();
        (ranked, timings)
    }

    /// Puts today's date or the time after the first candidate, so the word the code also spells stays on top.
    fn insert_shortcuts(&self, input: &str, ranked: &mut Vec<Candidate>) {
        let Some(time) = self.clock.as_ref().and_then(|clock| clock.now()) else {
            return;
        };
        let score = ranked.first().map_or(0.0, |first| first.score);
        let mut at = ranked.len().min(1);
        for text in shortcut::expand(input, &time) {
            if ranked.iter().any(|candidate| candidate.text == text) {
                continue;
            }
            ranked.insert(
                at,
                Candidate {
                    text,
                    source: CandidateSource::Shortcut,
                    score,
                },
            );
            at += 1;
        }
    }
}
