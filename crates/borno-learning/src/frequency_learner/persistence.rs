//! Loading and saving the learner's files: `user-words.tsv`, `user-choices.tsv` and `user-ngram.tsv`.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use borno_core::storage::{read_text_lossy, write_atomic};

use super::FrequencyLearner;
use crate::decay::{COLD_DAYS, Decay, today};
use crate::error::LearningError;
use crate::user_ngram::UserNgram;

const WORDS_FILE: &str = "user-words.tsv";
const CHOICES_FILE: &str = "user-choices.tsv";
const NGRAM_FILE: &str = "user-ngram.tsv";

impl FrequencyLearner {
    /// Opens the learner stored in `directory`, creating nothing until the first `flush`. Missing files mean a
    /// fresh start; malformed lines are skipped with a warning.
    pub fn open(directory: impl Into<PathBuf>) -> Result<Self, LearningError> {
        let directory = directory.into();
        let mut learner = Self::default();
        let now = today();
        learner.today.set(now);

        if let Some(text) = read_optional(&directory.join(WORDS_FILE))? {
            for line in data_lines(&text) {
                let fields: Vec<&str> = line.split('\t').collect();
                if let [word, count, day] = fields.as_slice()
                    && let (Ok(count), Ok(day)) = (count.parse::<u32>(), day.parse::<u64>())
                    && count > 0
                {
                    learner
                        .counts
                        .insert((*word).to_owned(), Decay::from_parts(count, day));
                } else {
                    tracing::warn!(file = WORDS_FILE, line, "skipped a malformed line");
                }
            }
        }
        if let Some(text) = read_optional(&directory.join(CHOICES_FILE))? {
            for line in data_lines(&text) {
                let fields: Vec<&str> = line.split('\t').collect();
                if let [input, word, count, day] = fields.as_slice()
                    && let (Ok(count), Ok(day)) = (count.parse::<u32>(), day.parse::<u64>())
                    && count > 0
                {
                    learner
                        .choices
                        .entry((*input).to_owned())
                        .or_default()
                        .insert((*word).to_owned(), Decay::from_parts(count, day));
                } else {
                    tracing::warn!(file = CHOICES_FILE, line, "skipped a malformed line");
                }
            }
        }
        if let Some(text) = read_optional(&directory.join(NGRAM_FILE))? {
            let (ngram, bad) = UserNgram::parse_lenient(&text);
            if bad > 0 {
                tracing::warn!(file = NGRAM_FILE, bad, "skipped malformed lines");
            }
            learner.ngram = ngram;
        }
        learner.directory = Some(directory);
        Ok(learner)
    }

    /// Writes the three files atomically when something changed. Entries unused for [`COLD_DAYS`] are dropped.
    /// Does nothing for an in-memory learner.
    pub fn save(&mut self) -> io::Result<()> {
        let Some(directory) = self.directory.clone() else {
            return Ok(());
        };
        if !self.dirty {
            return Ok(());
        }
        fs::create_dir_all(&directory)?;
        let now = today();
        self.today.set(now);
        let warm = |entry: &Decay| now.saturating_sub(entry.last_used) <= COLD_DAYS;

        self.counts.retain(|_, entry| warm(entry));
        self.choices.retain(|_, texts| {
            texts.retain(|_, entry| warm(entry));
            !texts.is_empty()
        });

        let mut words: Vec<_> = self.counts.iter().collect();
        words.sort_by(|a, b| a.0.cmp(b.0));
        write_atomic(&directory.join(WORDS_FILE), |out| {
            for (word, entry) in words {
                writeln!(out, "{word}\t{}\t{}", entry.count, entry.last_used)?;
            }
            Ok(())
        })?;

        let mut choices: Vec<_> = self
            .choices
            .iter()
            .flat_map(|(input, texts)| texts.iter().map(move |(text, entry)| (input, text, entry)))
            .collect();
        choices.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
        write_atomic(&directory.join(CHOICES_FILE), |out| {
            for (input, text, entry) in choices {
                writeln!(out, "{input}\t{text}\t{}\t{}", entry.count, entry.last_used)?;
            }
            Ok(())
        })?;

        let ngram = self.ngram.to_tsv();
        write_atomic(&directory.join(NGRAM_FILE), |out| {
            out.write_all(ngram.as_bytes())
        })?;
        self.dirty = false;
        Ok(())
    }
}

/// File contents, or `None` when the file does not exist.
fn read_optional(path: &Path) -> Result<Option<String>, LearningError> {
    Ok(read_text_lossy(path)?)
}

fn data_lines(source: &str) -> impl Iterator<Item = &str> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}
