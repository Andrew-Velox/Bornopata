//! The personal bigram: which word the user committed after which.

use foldhash::HashMap;

/// Context key for the first word of a sentence.
const SENTENCE_START: &str = "<s>";

/// Above this many (previous, word) pairs every count is halved and zeros are dropped, so old habits fade.
const MAX_PAIRS: usize = 200_000;

/// Counts of (previous word, word) transitions. Persisted as TSV rows `previous\tword\tcount`.
#[derive(Debug, Default, Clone)]
pub struct UserNgram {
    pairs: HashMap<String, HashMap<String, u32>>,
    pair_total: usize,
}

impl UserNgram {
    /// Counts `word` committed after `previous` (`None` at the start of a sentence).
    pub fn record(&mut self, previous: Option<&str>, word: &str) {
        if word.is_empty() {
            return;
        }
        let key = previous.unwrap_or(SENTENCE_START);
        let followers = self.pairs.entry(key.to_owned()).or_default();
        let count = followers.entry(word.to_owned()).or_default();
        if *count == 0 {
            self.pair_total += 1;
        }
        *count = count.saturating_add(1);
        if self.pair_total > MAX_PAIRS {
            self.halve();
        }
    }

    /// How often `word` followed `previous`.
    pub fn pair(&self, previous: Option<&str>, word: &str) -> u32 {
        self.pairs
            .get(previous.unwrap_or(SENTENCE_START))
            .and_then(|followers| followers.get(word))
            .copied()
            .unwrap_or(0)
    }

    /// Words that followed `previous`, most frequent first (ties by text), at most `limit`.
    pub fn followers(&self, previous: Option<&str>, limit: usize) -> Vec<(String, u32)> {
        let Some(followers) = self.pairs.get(previous.unwrap_or(SENTENCE_START)) else {
            return Vec::new();
        };
        let mut ranked: Vec<(String, u32)> = followers
            .iter()
            .map(|(word, count)| (word.clone(), *count))
            .collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        ranked.truncate(limit);
        ranked
    }

    pub fn is_empty(&self) -> bool {
        self.pair_total == 0
    }

    fn halve(&mut self) {
        for followers in self.pairs.values_mut() {
            followers.retain(|_, count| {
                *count /= 2;
                *count > 0
            });
        }
        self.pairs.retain(|_, followers| !followers.is_empty());
        self.pair_total = self.pairs.values().map(HashMap::len).sum();
    }

    /// Parses TSV rows; blank lines and `#` comments are skipped. Returns the table and the number of bad rows,
    /// which are dropped rather than failing the whole file: a crash can leave a line or two damaged.
    pub fn parse_lenient(source: &str) -> (Self, usize) {
        let mut ngram = Self::default();
        let mut bad = 0;
        for line in source.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            let [previous, word, count] = fields.as_slice() else {
                bad += 1;
                continue;
            };
            let Ok(count) = count.trim().parse::<u32>() else {
                bad += 1;
                continue;
            };
            if count == 0 || word.is_empty() || previous.is_empty() {
                continue;
            }
            let followers = ngram.pairs.entry((*previous).to_owned()).or_default();
            let slot = followers.entry((*word).to_owned()).or_default();
            if *slot == 0 {
                ngram.pair_total += 1;
            }
            *slot = slot.saturating_add(count);
        }
        (ngram, bad)
    }

    /// Serializes to TSV, sorted so the file diffs well.
    pub fn to_tsv(&self) -> String {
        let mut rows: Vec<(&str, &str, u32)> = self
            .pairs
            .iter()
            .flat_map(|(previous, followers)| {
                followers
                    .iter()
                    .map(move |(word, count)| (previous.as_str(), word.as_str(), *count))
            })
            .collect();
        rows.sort_unstable();
        rows.into_iter()
            .map(|(previous, word, count)| format!("{previous}\t{word}\t{count}\n"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_pairs_and_ranks_followers() {
        let mut ngram = UserNgram::default();
        ngram.record(None, "আমি");
        ngram.record(Some("আমি"), "বাংলা");
        ngram.record(Some("আমি"), "বাংলা");
        ngram.record(Some("আমি"), "ভাত");
        assert_eq!(ngram.pair(Some("আমি"), "বাংলা"), 2);
        assert_eq!(ngram.pair(None, "আমি"), 1);
        assert_eq!(ngram.pair(Some("আমি"), "নেই"), 0);
        let followers = ngram.followers(Some("আমি"), 5);
        assert_eq!(followers, [("বাংলা".to_owned(), 2), ("ভাত".to_owned(), 1)]);
        assert_eq!(ngram.followers(Some("আমি"), 1).len(), 1);
        assert!(ngram.followers(Some("অজানা"), 5).is_empty());
    }

    #[test]
    fn tsv_round_trip_skips_bad_rows() {
        let mut ngram = UserNgram::default();
        ngram.record(None, "আমি");
        ngram.record(Some("আমি"), "বাংলা");
        let text = ngram.to_tsv() + "broken line\nx\ty\tnot-a-number\n# comment\n";
        let (loaded, bad) = UserNgram::parse_lenient(&text);
        assert_eq!(bad, 2);
        assert_eq!(loaded.pair(Some("আমি"), "বাংলা"), 1);
        assert_eq!(loaded.pair(None, "আমি"), 1);
    }

    #[test]
    fn halving_drops_pairs_seen_once() {
        let mut ngram = UserNgram::default();
        ngram.record(Some("a"), "b");
        ngram.record(Some("a"), "c");
        ngram.record(Some("a"), "c");
        ngram.halve();
        assert_eq!(ngram.pair(Some("a"), "b"), 0);
        assert_eq!(ngram.pair(Some("a"), "c"), 1);
        assert_eq!(ngram.pair_total, 1);
    }
}
