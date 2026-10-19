//! Where the time of one candidate generation went.

use std::time::Duration;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Timings {
    /// The direct Avro conversion.
    pub direct: Duration,

    /// The word source lookup.
    pub dictionary: Duration,

    /// Learned choices, scoring and sorting.
    pub ranking: Duration,
}

impl Timings {
    pub fn total(&self) -> Duration {
        self.direct + self.dictionary + self.ranking
    }
}
