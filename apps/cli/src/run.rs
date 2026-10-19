//! Types the given words through the engine and reports every step.

use std::io::Write;

use borno_core::{Engine, Typed};

use crate::args::Args;
use crate::display::print_candidates;
use crate::error::CliError;

/// Runs `args` against `engine`, writing the report to `out`. Returns the committed text.
pub fn run(engine: &mut Engine, args: &Args, out: &mut impl Write) -> Result<String, CliError> {
    let mut committed = String::new();
    let input = args.words.join(" ");
    for ch in input.chars().chain(Some(' ')) {
        if !engine.composition().is_empty() && !engine.composition().accepts(ch) {
            show_and_pick(engine, args, out)?;
        }
        if let Typed::Commit(text) = engine.type_char(ch) {
            committed.push_str(&text);
        }
    }
    let committed = committed.trim_end().to_owned();
    writeln!(out, "text: {committed}")?;
    let next = engine.next_words(args.next);
    if !next.is_empty() {
        writeln!(out, "next: {}", next.join("  "))?;
    }
    engine.flush()?;
    Ok(committed)
}

fn show_and_pick(engine: &mut Engine, args: &Args, out: &mut impl Write) -> Result<(), CliError> {
    print_candidates(
        out,
        engine.composition().as_str(),
        engine.candidates(),
        engine.timings(),
        args.limit,
    )?;
    let count = engine.candidates().len();
    if args.pick == 0 || args.pick > count {
        return Err(CliError::PickOutOfRange {
            pick: args.pick,
            count,
        });
    }
    engine.highlight(args.pick - 1);
    Ok(())
}
