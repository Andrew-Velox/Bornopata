//! Builds the dictionary file: counts the words of the corpus, filters them and writes the `.bnd`.

mod args;
mod counter;
mod error;
mod report;
mod wikitext;

use borno_dictionary::DictionaryBuilder;
use borno_format::Metadata;
use clap::Parser;

use crate::args::Args;
use crate::counter::Counter;
use crate::error::BuildError;

fn main() {
    if let Err(error) = run(&Args::parse()) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> Result<(), BuildError> {
    if args.corpus.is_empty() && args.counts.is_empty() {
        return Err(BuildError::NoInput);
    }
    let mut counter = Counter::new(args.wikitext);
    for path in &args.corpus {
        counter.add_path(path)?;
    }
    for path in &args.counts {
        counter.add_counts(path)?;
    }
    let tokens = counter.tokens;
    let rejected = counter.rejected;
    let unique = counter.unique();
    let words = counter.finish(args.min_count, args.max_words);
    if words.is_empty() {
        return Err(BuildError::Empty);
    }
    if args.stats {
        report::print(tokens, rejected, unique, &words);
    }
    let mut builder = DictionaryBuilder::new();
    for (word, count) in &words {
        builder.add(word, *count);
    }
    let metadata = Metadata {
        name: args.name.clone(),
        license: args.license.clone(),
        attribution: args.attribution.clone(),
        source: args.source.clone(),
        version: args.version.clone(),
        entries: 0,
        generator: format!("borno-dict-build {}", env!("CARGO_PKG_VERSION")),
    };
    builder.write(&args.output, &metadata)?;
    eprintln!("wrote {} words to {}", words.len(), args.output.display());
    Ok(())
}
