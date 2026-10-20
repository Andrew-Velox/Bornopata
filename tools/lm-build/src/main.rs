//! Builds the language model file: counts which word follows which in the corpus and writes the `.bnd`.

mod args;
mod error;
mod pair_counter;
mod vocabulary;

use borno_format::Metadata;
use borno_lm::BigramModel;
use clap::Parser;

use crate::args::Args;
use crate::error::BuildError;
use crate::pair_counter::PairCounter;
use crate::vocabulary::Vocabulary;

fn main() {
    if let Err(error) = run(&Args::parse()) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> Result<(), BuildError> {
    if args.corpus.is_empty() {
        return Err(BuildError::NoCorpus);
    }
    let vocabulary = Vocabulary::read(&args.counts, args.vocabulary)?;
    let mut counter = PairCounter::new(&vocabulary, args.wikitext);
    for path in &args.corpus {
        counter.add_path(path)?;
    }
    let (unigrams, bigrams) = counter.finish(args.min_count, args.successors);
    if bigrams.is_empty() {
        return Err(BuildError::Empty);
    }
    let model = BigramModel::parse(&unigrams, &bigrams)?;
    let metadata = Metadata {
        name: args.name.clone(),
        license: args.license.clone(),
        attribution: args.attribution.clone(),
        source: args.source.clone(),
        version: args.version.clone(),
        entries: 0,
        generator: format!("borno-lm-build {}", env!("CARGO_PKG_VERSION")),
    };
    model.write_bnd(&args.output, &metadata)?;
    eprintln!(
        "wrote {} words and {} pairs to {}",
        model.word_count(),
        model.bigram_count(),
        args.output.display()
    );
    Ok(())
}
