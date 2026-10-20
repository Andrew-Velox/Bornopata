//! Counting which word follows which in corpus text.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use borno_corpus_text::wikitext;
use borno_dictionary::{normalize, tokens};

use crate::error::BuildError;
use crate::vocabulary::{SENTENCE_START_ID, Vocabulary};

/// Counts of words and of word pairs, over the words of a [`Vocabulary`].
pub struct PairCounter<'a> {
    vocabulary: &'a Vocabulary,

    /// Occurrences of each word, index = id; id 0 counts sentences.
    words: Vec<u64>,

    /// Pairs as `previous << 32 | word`.
    pairs: HashMap<u64, u32>,

    /// Strip wiki markup from each line first.
    wikitext: bool,
}

impl<'a> PairCounter<'a> {
    pub fn new(vocabulary: &'a Vocabulary, wikitext: bool) -> Self {
        Self {
            vocabulary,
            words: vec![0; vocabulary.len() + 1],
            pairs: HashMap::new(),
            wikitext,
        }
    }

    /// Counts the sentences of one line. A sentence ends at a danda, a question or exclamation mark, a full stop
    /// or the end of the line; a word outside the vocabulary breaks the chain, so no pair spans it.
    pub fn add_line(&mut self, line: &str) {
        if self.wikitext {
            return self.add_cleaned(&wikitext::strip(line));
        }
        self.add_cleaned(line);
    }

    fn add_cleaned(&mut self, line: &str) {
        for sentence in line.split(['।', '?', '!', '.', '\n']) {
            let mut previous = SENTENCE_START_ID;
            let mut started = false;
            for token in tokens(sentence) {
                let id = normalize(token).and_then(|word| self.vocabulary.id(&word));
                let Some(id) = id else {
                    previous = u32::MAX;
                    started = true;
                    continue;
                };
                if !started {
                    self.words[SENTENCE_START_ID as usize] += 1;
                    started = true;
                }
                self.words[id as usize] += 1;
                if previous != u32::MAX {
                    *self
                        .pairs
                        .entry(u64::from(previous) << 32 | u64::from(id))
                        .or_default() += 1;
                }
                previous = id;
            }
        }
    }

    pub fn add_reader(&mut self, reader: impl Read) -> std::io::Result<()> {
        let mut reader = BufReader::new(reader);
        let mut line = Vec::new();
        while reader.read_until(b'\n', &mut line)? > 0 {
            self.add_line(&String::from_utf8_lossy(&line));
            line.clear();
        }
        Ok(())
    }

    /// Counts a file, a directory tree or (for `-`) standard input.
    pub fn add_path(&mut self, path: &Path) -> Result<(), BuildError> {
        let read = |source| BuildError::Read {
            path: path.to_owned(),
            source,
        };
        if path == Path::new("-") {
            return self.add_reader(std::io::stdin().lock()).map_err(read);
        }
        if path.is_dir() {
            let mut files: Vec<PathBuf> = Vec::new();
            collect_files(path, &mut files).map_err(read)?;
            files.sort();
            for file in files {
                self.add_path(&file)?;
            }
            return Ok(());
        }
        self.add_reader(File::open(path).map_err(read)?)
            .map_err(read)
    }

    /// The model as two TSV texts, `word<TAB>count` and `previous<TAB>word<TAB>count`. Pairs seen fewer than
    /// `min_count` times are dropped, and each word keeps its `successors` most frequent ones.
    pub fn finish(self, min_count: u32, successors: usize) -> (String, String) {
        let mut unigrams = format!("<s>\t{}\n", self.words[SENTENCE_START_ID as usize]);
        for id in 1..self.words.len() {
            if self.words[id] > 0 {
                unigrams.push_str(&format!(
                    "{}\t{}\n",
                    self.vocabulary.word(id as u32),
                    self.words[id]
                ));
            }
        }
        let mut by_previous: HashMap<u32, Vec<(u32, u32)>> = HashMap::new();
        for (pair, count) in self.pairs {
            if count >= min_count {
                by_previous
                    .entry((pair >> 32) as u32)
                    .or_default()
                    .push((pair as u32, count));
            }
        }
        let mut previous_ids: Vec<u32> = by_previous.keys().copied().collect();
        previous_ids.sort_unstable();
        let mut bigrams = String::new();
        for previous in previous_ids {
            let mut following = by_previous.remove(&previous).unwrap_or_default();
            following.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            following.truncate(successors);
            let name = |id: u32| {
                if id == SENTENCE_START_ID {
                    "<s>"
                } else {
                    self.vocabulary.word(id)
                }
            };
            for (word, count) in following {
                bigrams.push_str(&format!("{}\t{}\t{count}\n", name(previous), name(word)));
            }
        }
        (unigrams, bigrams)
    }
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vocabulary() -> Vocabulary {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let number = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "borno-lm-vocab-{}-{number}.tsv",
            std::process::id()
        ));
        std::fs::write(&path, "আমি\t9\nভাত\t8\nখাই\t7\n").unwrap();
        let vocabulary = Vocabulary::read(&path, 10).unwrap();
        std::fs::remove_file(path).unwrap();
        vocabulary
    }

    #[test]
    fn pairs_follow_sentences_and_stop_at_unknown_words() {
        let vocabulary = vocabulary();
        let mut counter = PairCounter::new(&vocabulary, false);
        counter.add_line("আমি ভাত খাই। আমি ভাত খাই। আমি রুটি ভাত।");
        let (unigrams, bigrams) = counter.finish(1, 5);
        assert!(unigrams.starts_with("<s>\t3\n"));
        assert!(unigrams.contains("আমি\t3\n"));
        assert!(bigrams.contains("<s>\tআমি\t3\n"));
        assert!(bigrams.contains("আমি\tভাত\t2\n"));
        assert!(bigrams.contains("ভাত\tখাই\t2\n"));
        assert!(!bigrams.contains("রুটি"));
        // রুটি is unknown, so there is no আমি→রুটি→ভাত pair.
        assert!(!bigrams.contains("আমি\tভাত\t3"));
    }

    #[test]
    fn min_count_and_successor_limit_trim_the_pairs() {
        let vocabulary = vocabulary();
        let mut counter = PairCounter::new(&vocabulary, false);
        counter.add_line("আমি ভাত। আমি ভাত। আমি খাই।");
        let (_, bigrams) = counter.finish(2, 5);
        assert!(bigrams.contains("আমি\tভাত\t2\n"));
        assert!(!bigrams.contains("আমি\tখাই"));
    }
}
