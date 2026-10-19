//! The shared learner as an engine's own `Learner`.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use borno_core::Learner;
use borno_learning::FrequencyLearner;

/// The Router runs on one thread, so the lock is never contended; it only lets several engines hold the handle.
pub(super) struct SharedLearner(pub Arc<Mutex<FrequencyLearner>>);

impl SharedLearner {
    fn lock(&self) -> MutexGuard<'_, FrequencyLearner> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Learner for SharedLearner {
    fn record(&mut self, input: &str, text: &str, previous: Option<&str>) {
        self.lock().record(input, text, previous);
    }

    fn weight(&self, text: &str) -> u32 {
        self.lock().weight(text)
    }

    fn choice_weight(&self, input: &str, text: &str) -> u32 {
        self.lock().choice_weight(input, text)
    }

    fn choices(&self, input: &str) -> Vec<(String, u32)> {
        self.lock().choices(input)
    }

    fn pair_count(&self, previous: &str, text: &str) -> u32 {
        self.lock().pair_count(previous, text)
    }

    fn followers(&self, previous: Option<&str>, limit: usize) -> Vec<(String, u32)> {
        self.lock().followers(previous, limit)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.lock().save()
    }
}
