//! Candidate generation: direct conversion, word source suggestions and learned choices, ranked.

use std::collections::HashMap;
use std::time::Instant;

use super::ranking::Evidence;
use super::{Engine, Query, Timings};
use crate::candidate::Candidate;

/// Longer inputs get the direct conversion only: the word source is built for words, not paragraphs.
const MAX_SUGGEST_LEN: usize = 64;

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
            evidence.entry(direct).or_default().direct = true;
        }

        let started = Instant::now();
        let suggested = if input.len() <= MAX_SUGGEST_LEN {
            self.words.suggest(input)
        } else {
            Vec::new()
        };
        for text in suggested {
            let entry = evidence.entry(text.clone()).or_insert_with(|| {
                order.push(text);
                Evidence::default()
            });
            entry.dictionary = true;
        }
        timings.dictionary = started.elapsed();

        let started = Instant::now();
        for (text, _) in self.learner.choices(input) {
            evidence.entry(text.clone()).or_insert_with(|| {
                order.push(text);
                Evidence::default()
            });
        }
        let mut ranked: Vec<Candidate> = order
            .into_iter()
            .map(|text| {
                let mut known = evidence.remove(&text).unwrap_or_default();
                known.choice_weight = self.learner.choice_weight(input, &text);
                known.word_weight = self.learner.weight(&text);
                known.pair_count =
                    previous.map_or(0, |previous| self.learner.pair_count(previous, &text));
                known.log_prob = self.language_model.log_prob(previous, &text);
                Candidate {
                    source: known.source(),
                    score: known.score(),
                    text,
                }
            })
            .collect();
        ranked.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.text.cmp(&b.text))
        });
        ranked.truncate(self.candidate_limit);
        timings.ranking = started.elapsed();
        (ranked, timings)
    }
}
