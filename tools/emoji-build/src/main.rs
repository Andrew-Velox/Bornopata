//! Builds `assets/emoji/emoji.tsv`: every emoji with its Bangla and English name and keywords.
//! `borno-emoji-build --emoji-test emoji-test.txt --bn bn.xml --en en.xml --output assets/emoji/emoji.tsv`

mod annotations;
mod error;
mod sequences;

use std::path::{Path, PathBuf};

use clap::Parser;

use crate::annotations::Annotations;
use crate::error::BuildError;

#[derive(Debug, Parser)]
#[command(
    name = "borno-emoji-build",
    about = "Builds the emoji table from Unicode's emoji-test.txt and the CLDR annotations"
)]
struct Cli {
    /// `emoji-test.txt` from unicode.org/Public/emoji/latest.
    #[arg(long)]
    emoji_test: PathBuf,

    /// CLDR `common/annotations/bn.xml`.
    #[arg(long)]
    bn: PathBuf,

    /// CLDR `common/annotations/en.xml`.
    #[arg(long)]
    en: PathBuf,

    /// Emoji newer than this are left out because common fonts cannot draw them yet.
    #[arg(long, default_value = "15.1")]
    max_version: String,

    #[arg(long, short)]
    output: PathBuf,
}

fn read(path: &Path) -> Result<String, BuildError> {
    std::fs::read_to_string(path).map_err(|source| BuildError::Read {
        path: path.to_owned(),
        source,
    })
}

fn run(cli: &Cli) -> Result<(), BuildError> {
    let max_version = sequences::parse_version_arg(&cli.max_version)?;
    let order = sequences::read(&read(&cli.emoji_test)?, max_version);
    let bn = Annotations::parse(&read(&cli.bn)?);
    let en = Annotations::parse(&read(&cli.en)?);

    let mut out = String::from(
        "# Emoji with their names and keywords, in Unicode's order. Built by tools/emoji-build; see README.md.\n\
         # emoji<TAB>Bangla name<TAB>Bangla keywords<TAB>English name<TAB>English keywords (keywords separated by |)\n",
    );
    let mut rows = 0;
    for sequence in order {
        let (bn, en) = (bn.get(&sequence), en.get(&sequence));
        if bn.is_none() && en.is_none() {
            continue;
        }
        let (bn, en) = (
            bn.cloned().unwrap_or_default(),
            en.cloned().unwrap_or_default(),
        );
        out.push_str(&format!(
            "{sequence}\t{}\t{}\t{}\t{}\n",
            bn.name,
            bn.keywords.join("|"),
            en.name,
            en.keywords.join("|")
        ));
        rows += 1;
    }
    std::fs::write(&cli.output, out).map_err(|source| BuildError::Write {
        path: cli.output.clone(),
        source,
    })?;
    println!("wrote {rows} emoji to {}", cli.output.display());
    Ok(())
}

fn main() {
    if let Err(error) = run(&Cli::parse()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
