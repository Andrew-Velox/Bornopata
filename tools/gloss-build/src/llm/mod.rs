//! Filling the gaps Wiktionary leaves with a language model, only when a key is supplied.
//!
//! The model and endpoint are the user's: any OpenAI-compatible service with a free tier works. Nothing here runs
//! without `BORNO_LLM_API_KEY`; then the table stays Wiktionary-only and the other words have no gloss.

mod client;
mod prompt;
mod settings;

#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub use client::{Client, LlmError};
pub use settings::{Missing, Settings};

use crate::error::BuildError;
use crate::ranks::top;
use crate::table;

pub struct Options {
    pub counts: std::collections::HashMap<String, u64>,

    /// How many of the most frequent words to cover.
    pub top: usize,

    /// Tables whose words already have a gloss and are not asked for.
    pub existing: Vec<PathBuf>,

    /// The table the model's glosses are appended to; words already in it are not asked for again, so a stopped run
    /// continues where it ended.
    pub output: PathBuf,

    pub batch: usize,

    /// Pause between requests, to stay inside a free tier's rate limit.
    pub delay: Duration,

    /// Stop after this many requests.
    pub max_requests: Option<usize>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub asked: usize,
    pub glossed: usize,
    pub requests: usize,
}

const HEADER: &str = "Glosses written by a language model for words Wiktionary does not cover, by borno-gloss-build.\n\
    The model and provider are in the source column (llm:<model>); see assets/gloss/README.md.\n\
    word, part of speech, gloss, source";

/// Asks the model for the words of the top `options.top` that have no gloss yet, batch by batch, appending the answers
/// to `options.output` after every batch.
pub fn generate(client: &Client, options: &Options) -> Result<Report, BuildError> {
    let mut have: HashSet<String> = HashSet::new();
    for path in options
        .existing
        .iter()
        .chain(std::iter::once(&options.output))
    {
        if path.exists() {
            have.extend(table::read(path)?.into_iter().map(|entry| entry.word));
        }
    }
    let wanted: Vec<String> = top(&options.counts, options.top)
        .into_iter()
        .map(|(word, _)| word)
        .filter(|word| !have.contains(word))
        .collect();
    let mut report = Report::default();
    if !options.output.exists() {
        write_text(&options.output, &header_text())?;
    }
    for batch in wanted.chunks(options.batch.max(1)) {
        if options
            .max_requests
            .is_some_and(|max| report.requests >= max)
        {
            break;
        }
        if report.requests > 0 {
            std::thread::sleep(options.delay);
        }
        report.requests += 1;
        let reply = match client.complete(prompt::SYSTEM, &prompt::user_message(batch)) {
            Ok(reply) => reply,
            Err(error @ LlmError::Refused(_)) => return Err(BuildError::Llm(error.to_string())),
            Err(error) => {
                // A busy or broken endpoint after the retries: keep what is written; a later run continues.
                eprintln!("stopping: {error}");
                break;
            }
        };
        report.asked += batch.len();
        let answers = prompt::read_reply(&reply, batch);
        report.glossed += answers.len();
        let source = format!("llm:{}", client.model());
        let mut text = String::new();
        for answer in answers {
            text.push_str(&format!(
                "{}\t{}\t{}\t{source}\n",
                answer.word, answer.pos, answer.text
            ));
        }
        append_text(&options.output, &text)?;
        eprintln!(
            "{} of {} words asked, {} glossed",
            report.asked,
            wanted.len(),
            report.glossed
        );
    }
    Ok(report)
}

fn header_text() -> String {
    HEADER.lines().map(|line| format!("# {line}\n")).collect()
}

fn write_text(path: &Path, text: &str) -> Result<(), BuildError> {
    std::fs::write(path, text).map_err(|source| BuildError::Write {
        path: path.to_owned(),
        source,
    })
}

fn append_text(path: &Path, text: &str) -> Result<(), BuildError> {
    let write = |error| BuildError::Write {
        path: path.to_owned(),
        source: error,
    };
    std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .map_err(write)
}
