//! Command-line arguments.

use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "borno-lm-build",
    about = "Builds the Bornopata language model from corpus text: which word follows which. The corpus is read, never copied."
)]
pub struct Args {
    /// Where to write the `.bnd` language model.
    #[arg(long, short)]
    pub output: PathBuf,

    /// Corpus text: a file, a directory (every file inside, recursively) or `-` for standard input, one sentence or
    /// more per line. Repeatable. Decompress first, for example `bzcat dump.xml.bz2 | borno-lm-build --corpus - ...`.
    #[arg(long)]
    pub corpus: Vec<PathBuf>,

    /// Strip wiki markup from the corpus lines first. Use it for a Wikipedia dump.
    #[arg(long)]
    pub wikitext: bool,

    /// The dictionary's `word<TAB>count` file, most frequent first (`borno-dict-build --dump-counts`). Only its
    /// first `--vocabulary` words are in the model, so the model and the dictionary spell alike.
    #[arg(long)]
    pub counts: PathBuf,

    /// How many of the most frequent words the model knows.
    #[arg(long, default_value_t = 30_000)]
    pub vocabulary: usize,

    /// Drop word pairs seen fewer times than this.
    #[arg(long, default_value_t = 10)]
    pub min_count: u32,

    /// Keep this many likely successors per word; the file grows with it.
    #[arg(long, default_value_t = 16)]
    pub successors: usize,

    /// Name stored in the file.
    #[arg(long, default_value = "Bornopata word pairs")]
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
}
