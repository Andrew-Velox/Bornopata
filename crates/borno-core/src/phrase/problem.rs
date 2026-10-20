//! A line of a phrase file that could not be used.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhraseProblem {
    /// 1-based line number in the file.
    pub line: usize,

    pub reason: &'static str,
}

impl fmt::Display for PhraseProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.reason)
    }
}
