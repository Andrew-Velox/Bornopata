//! Text output of candidates and timings.

use std::io::Write;
use std::time::Duration;

use borno_core::{Candidate, Timings};

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

/// Prints one word's candidates with the source and score of each, and where the time went.
pub fn print_candidates(
    out: &mut impl Write,
    input: &str,
    candidates: &[Candidate],
    timings: Timings,
    limit: usize,
) -> std::io::Result<()> {
    writeln!(
        out,
        "{input}  ({:.3} ms: direct {:.3}, dictionary {:.3}, ranking {:.3})",
        millis(timings.total()),
        millis(timings.direct),
        millis(timings.dictionary),
        millis(timings.ranking),
    )?;
    if candidates.is_empty() {
        return writeln!(out, "  (no candidates)");
    }
    for (index, candidate) in candidates.iter().take(limit).enumerate() {
        writeln!(
            out,
            "  {}. {}  [{}]  {:.2}",
            index + 1,
            candidate.text,
            candidate.source,
            candidate.score
        )?;
    }
    if candidates.len() > limit {
        writeln!(out, "  ... {} more", candidates.len() - limit)?;
    }
    Ok(())
}
