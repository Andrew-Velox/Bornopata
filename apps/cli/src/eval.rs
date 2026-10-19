//! Measures the candidate list against a file of typed spellings and the words they should give.

use std::io::Write;
use std::path::Path;

use borno_core::Engine;

use crate::error::CliError;

/// Ranks that are reported: how often the word is first, among the first three and among the first nine.
const CUTOFFS: [usize; 3] = [1, 3, 9];

/// Reads `roman<TAB>word|word` lines (`#` starts a comment), queries each and writes the hit rates and the misses.
pub fn evaluate(engine: &Engine, path: &Path, out: &mut impl Write) -> Result<(), CliError> {
    let text = std::fs::read_to_string(path).map_err(|source| CliError::Eval {
        path: path.to_owned(),
        source,
    })?;
    let mut hits = [0usize; CUTOFFS.len()];
    let mut total = 0usize;
    let mut misses = Vec::new();
    for line in text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let Some((roman, accepted)) = line.split_once('\t') else {
            continue;
        };
        // The file may hold nukta letters in either Unicode form; compare in the form the dictionary uses.
        let accepted: Vec<String> = accepted
            .split('|')
            .map(|word| borno_dictionary::normalize(word).unwrap_or_else(|| word.to_owned()))
            .collect();
        let query = engine.query(roman);
        let rank = query
            .candidates
            .iter()
            .position(|candidate| accepted.contains(&candidate.text))
            .map(|index| index + 1);
        total += 1;
        for (hit, cutoff) in hits.iter_mut().zip(CUTOFFS) {
            if rank.is_some_and(|rank| rank <= cutoff) {
                *hit += 1;
            }
        }
        if rank.is_none_or(|rank| rank > 1) {
            let shown: Vec<&str> = query
                .candidates
                .iter()
                .take(3)
                .map(|c| c.text.as_str())
                .collect();
            misses.push((roman.to_owned(), accepted.join("|"), rank, shown.join(" ")));
        }
    }
    for (hit, cutoff) in hits.iter().zip(CUTOFFS) {
        writeln!(
            out,
            "top {cutoff}: {hit}/{total} = {:.1}%",
            100.0 * *hit as f64 / total.max(1) as f64
        )?;
    }
    for (roman, expected, rank, shown) in misses {
        let rank = rank.map_or("-".to_owned(), |rank| rank.to_string());
        writeln!(
            out,
            "  {roman:<18} want {expected:<14} rank {rank:<3} got {shown}"
        )?;
    }
    Ok(())
}
