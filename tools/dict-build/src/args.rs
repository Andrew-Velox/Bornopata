//! Command-line arguments.

use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "borno-dict-build",
    about = "Builds the Bornopata dictionary from corpus text or word counts. The corpus is read, never copied."
)]
pub struct Args {
    /// Where to write the `.bnd` dictionary.
    #[arg(long, short)]
    pub output: PathBuf,

    /// Corpus text: a file, a directory (every file inside, recursively) or `-` for standard input.
    /// Any text works, including raw wiki XML; only runs of Bengali letters are used. Repeatable.
    /// Decompress first, for example `bzcat dump.xml.bz2 | borno-dict-build --corpus - ...`.
    #[arg(long)]
    pub corpus: Vec<PathBuf>,

    /// Strip wiki markup (templates, parameter names, category and file links, tags) from the corpus lines before
    /// counting. Use it for Wikipedia dumps; without it names like `|তারিখ=` count as words millions of times.
    #[arg(long)]
    pub wikitext: bool,

    /// A `word<TAB>count` file added to the counts. Repeatable.
    #[arg(long)]
    pub counts: Vec<PathBuf>,

    /// Drop words seen fewer times than this in total.
    #[arg(long, default_value_t = 3)]
    pub min_count: u32,

    /// Keep only this many of the most frequent words.
    #[arg(long)]
    pub max_words: Option<usize>,

    /// Name stored in the file, shown wherever the dictionary is listed.
    #[arg(long, default_value = "Bornopata word list")]
    pub name: String,

    /// SPDX license of the data, for example `CC-BY-SA-4.0`.
    #[arg(long)]
    pub license: String,

    /// Attribution line the license asks for.
    #[arg(long, default_value = "")]
    pub attribution: String,

    /// Where the corpus came from.
    #[arg(long, default_value = "")]
    pub source: String,

    /// Version of the data, such as the corpus date.
    #[arg(long, default_value = "")]
    pub version: String,

    /// Print the most frequent words and rough coverage.
    #[arg(long)]
    pub stats: bool,
}
