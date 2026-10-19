//! Test doubles: an in-memory learner and language model, and engine builders.

use std::collections::HashMap;

use borno_transliteration::{FixedWordSource, UpodeshWordSource};

use crate::{Engine, LanguageModel, Learner};

/// Engine over the upodesh scaffold list, the same path the CLI takes.
pub fn scaffold_engine() -> Engine {
    Engine::new(Box::new(UpodeshWordSource::new()))
}

/// Engine over a fixed list: roman input mapped to words.
pub fn fixed_engine(entries: &[(&str, &[&str])]) -> Engine {
    let entries = entries.iter().map(|(roman, words)| {
        (
            (*roman).to_owned(),
            words.iter().map(|word| (*word).to_owned()).collect(),
        )
    });
    Engine::new(Box::new(FixedWordSource::new(entries)))
}

/// A learner that keeps everything in maps, enough to exercise the engine.
#[derive(Default)]
pub struct MemoryLearner {
    words: HashMap<String, u32>,
    choices: HashMap<(String, String), u32>,
    pairs: HashMap<(Option<String>, String), u32>,
}

impl Learner for MemoryLearner {
    fn record(&mut self, input: &str, text: &str, previous: Option<&str>) {
        *self.words.entry(text.to_owned()).or_default() += 1;
        *self
            .choices
            .entry((input.to_owned(), text.to_owned()))
            .or_default() += 1;
        *self
            .pairs
            .entry((previous.map(str::to_owned), text.to_owned()))
            .or_default() += 1;
    }

    fn weight(&self, text: &str) -> u32 {
        self.words.get(text).copied().unwrap_or(0)
    }

    fn choice_weight(&self, input: &str, text: &str) -> u32 {
        self.choices
            .get(&(input.to_owned(), text.to_owned()))
            .copied()
            .unwrap_or(0)
    }

    fn choices(&self, input: &str) -> Vec<(String, u32)> {
        let mut found: Vec<(String, u32)> = self
            .choices
            .iter()
            .filter(|((typed, _), _)| typed == input)
            .map(|((_, text), count)| (text.clone(), *count))
            .collect();
        found.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        found
    }

    fn pair_count(&self, previous: &str, text: &str) -> u32 {
        self.pairs
            .get(&(Some(previous.to_owned()), text.to_owned()))
            .copied()
            .unwrap_or(0)
    }

    fn followers(&self, previous: Option<&str>, limit: usize) -> Vec<(String, u32)> {
        let mut found: Vec<(String, u32)> = self
            .pairs
            .iter()
            .filter(|((before, _), _)| before.as_deref() == previous)
            .map(|((_, text), count)| (text.clone(), *count))
            .collect();
        found.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        found.truncate(limit);
        found
    }
}

/// A language model with fixed successors per previous word.
pub struct FixedModel(pub Vec<(&'static str, Vec<&'static str>)>);

impl LanguageModel for FixedModel {
    fn log_prob(&self, previous: Option<&str>, word: &str) -> Option<f64> {
        let (_, followers) = self
            .0
            .iter()
            .find(|(before, _)| Some(*before) == previous)?;
        followers
            .iter()
            .position(|follower| *follower == word)
            .map(|rank| -1.0 - 4.0 * rank as f64)
    }

    fn successors(&self, previous: Option<&str>, limit: usize) -> Vec<(String, f64)> {
        let Some((_, followers)) = self.0.iter().find(|(before, _)| Some(*before) == previous)
        else {
            return Vec::new();
        };
        followers
            .iter()
            .take(limit)
            .enumerate()
            .map(|(rank, word)| ((*word).to_owned(), -1.0 - 4.0 * rank as f64))
            .collect()
    }
}
