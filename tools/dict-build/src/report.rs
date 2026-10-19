//! The summary printed with `--stats`.

pub fn print(tokens: u64, rejected: u64, unique: usize, kept: &[(String, u32)]) {
    let total: u64 = kept.iter().map(|(_, count)| u64::from(*count)).sum();
    eprintln!("Bengali tokens: {tokens} ({rejected} malformed and dropped)");
    eprintln!("distinct words: {unique}, kept: {}", kept.len());
    eprintln!("occurrences covered by the kept words: {total}");
    for (cut, label) in [
        (1_000, "top 1000"),
        (10_000, "top 10000"),
        (100_000, "top 100000"),
    ] {
        if kept.len() >= cut {
            let covered: u64 = kept[..cut].iter().map(|(_, count)| u64::from(*count)).sum();
            eprintln!(
                "{label}: {:.1}% of kept occurrences",
                100.0 * covered as f64 / total as f64
            );
        }
    }
    for (word, count) in kept.iter().take(20) {
        eprintln!("  {count:>10}  {word}");
    }
}
