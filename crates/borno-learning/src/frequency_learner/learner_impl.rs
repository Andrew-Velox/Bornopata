//! `Learner` for `FrequencyLearner`.

use borno_core::Learner;

use super::FrequencyLearner;
use crate::decay::{Decay, today};

impl Learner for FrequencyLearner {
    fn record(&mut self, input: &str, text: &str, previous: Option<&str>) {
        if text.is_empty() {
            return;
        }
        let now = today();
        self.today.set(now);
        self.counts
            .entry(text.to_owned())
            .or_insert_with(|| Decay::from_parts(0, now))
            .record(1, now);
        if !input.is_empty() {
            self.choices
                .entry(input.to_owned())
                .or_default()
                .entry(text.to_owned())
                .or_insert_with(|| Decay::from_parts(0, now))
                .record(1, now);
        }
        self.ngram.record(previous, text);
        self.dirty = true;
    }

    fn weight(&self, text: &str) -> u32 {
        self.counts
            .get(text)
            .map_or(0, |entry| self.effective(entry))
    }

    fn choice_weight(&self, input: &str, text: &str) -> u32 {
        self.choices
            .get(input)
            .and_then(|texts| texts.get(text))
            .map_or(0, |entry| self.effective(entry))
    }

    fn choices(&self, input: &str) -> Vec<(String, u32)> {
        let Some(texts) = self.choices.get(input) else {
            return Vec::new();
        };
        let mut ranked: Vec<(String, u32)> = texts
            .iter()
            .map(|(text, entry)| (text.clone(), self.effective(entry)))
            .filter(|(_, weight)| *weight > 0)
            .collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        ranked
    }

    fn pair_count(&self, previous: &str, text: &str) -> u32 {
        self.ngram.pair(Some(previous), text)
    }

    fn followers(&self, previous: Option<&str>, limit: usize) -> Vec<(String, u32)> {
        self.ngram.followers(previous, limit)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.save()
    }
}
