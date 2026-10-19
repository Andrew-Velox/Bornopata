//! Command-line arguments.

use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "borno-cli", about = "Bornopata Core test tool")]
pub struct Args {
    /// Roman words to type, in order, through the same engine path the input method uses.
    /// Punctuation inside a word ends it (`ami.` commits আমি then ।).
    pub words: Vec<String>,

    /// Candidates to show per word.
    #[arg(long, default_value_t = 9)]
    pub limit: usize,

    /// Commit candidate N (1-based) for every word instead of the first. With --user-dir the choice is learned.
    #[arg(long, default_value_t = 1)]
    pub pick: usize,

    /// Directory for learned data. Without it learning lasts for this run only.
    #[arg(long)]
    pub user_dir: Option<PathBuf>,

    /// Dictionary file. Without it `$BORNOPATA_DICT` or `words.bnd` in the user data directory is used if present,
    /// else only the built-in scaffold word list answers.
    #[arg(long)]
    pub dict: Option<PathBuf>,

    /// Ignore any dictionary file and use only the built-in scaffold word list.
    #[arg(long, conflicts_with = "dict")]
    pub no_dict: bool,

    /// Language model as a `.bnd` file.
    #[arg(long, conflicts_with_all = ["lm_unigram", "lm_bigram"])]
    pub lm: Option<PathBuf>,

    /// Language model unigram TSV (`word\tcount`); needs --lm-bigram.
    #[arg(long)]
    pub lm_unigram: Option<PathBuf>,

    /// Language model bigram TSV (`previous\tword\tcount`); needs --lm-unigram.
    #[arg(long)]
    pub lm_bigram: Option<PathBuf>,

    /// Measure the candidate lists instead of typing: a file of `roman<TAB>word|word` lines. Prints how often the
    /// word is first, in the first three and in the first nine, and lists the misses.
    #[arg(long)]
    pub eval: Option<PathBuf>,

    /// Next-word suggestions to show after the last word.
    #[arg(long, default_value_t = 5)]
    pub next: usize,
}
