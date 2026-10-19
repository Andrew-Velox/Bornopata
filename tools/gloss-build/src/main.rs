//! Builds the Bornopata gloss file. `wiktionary` cleans Wiktionary entries into a gloss table, `sheet` writes the review
//! sheet, `export` writes the `.bnd` the input method loads. Design: `docs/design/glosses.md`.

mod clean;
mod error;
mod export;
mod kaikki;
mod llm;
mod prefer;
mod ranks;
mod sheet;
mod table;

use std::path::PathBuf;

use borno_format::Metadata;
use clap::{Parser, Subcommand};

use crate::clean::{Cleaner, Route};
use crate::error::BuildError;
use crate::kaikki::Index;
use crate::prefer::Prefer;
use crate::table::Entry;

#[derive(Debug, Parser)]
#[command(
    name = "borno-gloss-build",
    about = "Builds the Bornopata gloss file from Wiktionary and, optionally, a language model"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Cleans the Kaikki extract of English Wiktionary into a gloss table for the most frequent words.
    Wiktionary {
        /// The Kaikki Bengali JSONL (kaikki.org/dictionary/Bengali).
        #[arg(long)]
        kaikki: PathBuf,

        /// A `word<TAB>count` file; its most frequent words are glossed.
        #[arg(long)]
        counts: PathBuf,

        /// How many of the most frequent words to gloss.
        #[arg(long, default_value_t = 5000)]
        top: usize,

        /// Which entry to use for words with several parts of speech: `word<TAB>pos` lines.
        #[arg(long)]
        prefer: Option<PathBuf>,

        /// Where to write the gloss table.
        #[arg(long, short)]
        output: PathBuf,
    },

    /// Asks a language model for the words Wiktionary left without a gloss. Needs `BORNO_LLM_API_KEY`,
    /// `BORNO_LLM_BASE_URL` and `BORNO_LLM_MODEL` in the environment or in an ignored `.env` file; without a key it does
    /// nothing and the table stays Wiktionary-only. Resumable: run it again to continue.
    Generate {
        #[arg(long)]
        counts: PathBuf,

        #[arg(long, default_value_t = 5000)]
        top: usize,

        /// Gloss tables whose words are not asked for (the Wiktionary table). Repeatable.
        #[arg(long = "entries")]
        entries: Vec<PathBuf>,

        /// The table the model's glosses are appended to.
        #[arg(long, short)]
        output: PathBuf,

        /// Words per request.
        #[arg(long, default_value_t = 25)]
        batch: usize,

        /// Milliseconds between requests, to stay inside a free tier's rate limit.
        #[arg(long, default_value_t = 2500)]
        delay_ms: u64,

        /// Stop after this many requests (a free tier has a daily cap).
        #[arg(long)]
        max_requests: Option<usize>,
    },

    /// Writes a review sheet of the most frequent words next to their glosses.
    Sheet {
        /// A gloss table (`wiktionary` or `generate` output). Repeatable; a later file wins.
        #[arg(long = "entries", required = true)]
        entries: Vec<PathBuf>,

        #[arg(long)]
        counts: PathBuf,

        /// The Kaikki JSONL, to show Wiktionary's own first sense beside each gloss.
        #[arg(long)]
        kaikki: Option<PathBuf>,

        #[arg(long, default_value_t = 200)]
        rows: usize,

        #[arg(long, short)]
        output: PathBuf,
    },

    /// Writes the `.bnd` the input method loads from gloss tables.
    Export {
        /// A gloss table. Repeatable; a later file wins for a word both have.
        #[arg(long = "entries", required = true)]
        entries: Vec<PathBuf>,

        #[arg(long, short)]
        output: PathBuf,

        #[arg(long, default_value = "Bornopata English glosses")]
        name: String,

        /// SPDX license of the data.
        #[arg(long)]
        license: String,

        #[arg(long, default_value = "")]
        attribution: String,

        #[arg(long, default_value = "")]
        source: String,

        #[arg(long, default_value = "")]
        version: String,
    },
}

fn main() {
    if let Err(error) = run(Cli::parse().command) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(command: Command) -> Result<(), BuildError> {
    match command {
        Command::Wiktionary {
            kaikki,
            counts,
            top,
            prefer,
            output,
        } => wiktionary(&kaikki, &counts, top, prefer.as_deref(), &output),
        Command::Generate {
            counts,
            top,
            entries,
            output,
            batch,
            delay_ms,
            max_requests,
        } => generate(&counts, top, entries, output, batch, delay_ms, max_requests),
        Command::Sheet {
            entries,
            counts,
            kaikki,
            rows,
            output,
        } => {
            let counts = ranks::read_counts(&counts)?;
            let entries = read_all(&entries)?;
            let index = kaikki.as_deref().map(Index::load).transpose()?;
            sheet::write(&output, rows, &counts, &entries, index.as_ref())?;
            eprintln!("wrote {rows} rows to {}", output.display());
            Ok(())
        }
        Command::Export {
            entries,
            output,
            name,
            license,
            attribution,
            source,
            version,
        } => {
            let entries = read_all(&entries)?;
            let metadata = Metadata {
                name,
                license,
                attribution,
                source,
                version,
                entries: 0,
                generator: format!("borno-gloss-build {}", env!("CARGO_PKG_VERSION")),
            };
            let count = export::export(&entries, &output, &metadata)?;
            eprintln!("wrote {count} glosses to {}", output.display());
            Ok(())
        }
    }
}

fn generate(
    counts: &std::path::Path,
    top: usize,
    existing: Vec<PathBuf>,
    output: PathBuf,
    batch: usize,
    delay_ms: u64,
    max_requests: Option<usize>,
) -> Result<(), BuildError> {
    let dotenv = std::fs::read_to_string(".env").unwrap_or_default();
    let settings = match llm::Settings::resolve(|name| std::env::var(name).ok(), &dotenv) {
        Ok(settings) => settings,
        Err(llm::Missing::Key) => {
            eprintln!(
                "no BORNO_LLM_API_KEY in the environment or .env: skipping the language model; the words Wiktionary does not cover keep no gloss"
            );
            return Ok(());
        }
        Err(llm::Missing::Other(name)) => {
            return Err(BuildError::Llm(format!("{name} is not set")));
        }
    };
    let options = llm::Options {
        counts: ranks::read_counts(counts)?,
        top,
        existing,
        output,
        batch,
        delay: std::time::Duration::from_millis(delay_ms),
        max_requests,
    };
    let client = llm::Client::new(settings);
    let report = llm::generate(&client, &options)?;
    eprintln!(
        "{} requests, {} words asked, {} glossed",
        report.requests, report.asked, report.glossed
    );
    Ok(())
}

/// The tables merged: for a word in several, the last file wins.
fn read_all(paths: &[PathBuf]) -> Result<Vec<Entry>, BuildError> {
    let mut merged: std::collections::BTreeMap<String, Entry> = std::collections::BTreeMap::new();
    for path in paths {
        for entry in table::read(path)? {
            merged.insert(entry.word.clone(), entry);
        }
    }
    Ok(merged.into_values().collect())
}

fn wiktionary(
    kaikki: &std::path::Path,
    counts: &std::path::Path,
    top: usize,
    prefer: Option<&std::path::Path>,
    output: &std::path::Path,
) -> Result<(), BuildError> {
    let index = Index::load(kaikki)?;
    let counts = ranks::read_counts(counts)?;
    let prefer = prefer.map(Prefer::load).transpose()?.unwrap_or_default();
    let cleaner = Cleaner::new(&index, &prefer, &counts);
    let words = ranks::top(&counts, top);
    let mut entries = Vec::new();
    let (mut own, mut through) = (0usize, 0usize);
    for (word, _) in &words {
        let Some(resolved) = cleaner.resolve(word) else {
            continue;
        };
        let source = match &resolved.route {
            Route::Own => {
                own += 1;
                "wiktionary".to_owned()
            }
            Route::Lemma(lemma) => {
                through += 1;
                format!("wiktionary:{lemma}")
            }
        };
        entries.push(Entry {
            word: word.clone(),
            pos: resolved.pos.to_owned(),
            text: resolved.text,
            source,
        });
    }
    let header = "Glosses cleaned from English Wiktionary (via the Kaikki extract) by borno-gloss-build.\n\
        Wiktionary text is CC BY-SA 4.0 and GFDL; see assets/gloss/README.md for attribution.\n\
        word, part of speech, gloss, source (wiktionary, or wiktionary:<lemma> for an inflected form)";
    table::write(output, header, &entries)?;
    eprintln!(
        "{} of the {} most frequent words glossed ({own} from their own entry, {through} through a lemma); wrote {}",
        entries.len(),
        words.len(),
        output.display()
    );
    Ok(())
}
