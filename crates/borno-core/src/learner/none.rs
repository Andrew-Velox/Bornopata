//! A learner that remembers nothing.

use super::Learner;

#[derive(Debug, Default, Clone, Copy)]
pub struct NoLearner;

impl Learner for NoLearner {
    fn record(&mut self, _input: &str, _text: &str, _previous: Option<&str>) {}
}
