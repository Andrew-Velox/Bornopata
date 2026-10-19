//! Command-line tool for the Core: type Roman words through the engine and see the candidates.

mod args;
mod display;
mod error;
mod eval;
mod run;
mod setup;

use clap::Parser;

use crate::args::Args;
use crate::error::CliError;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();
    if let Err(error) = execute() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn execute() -> Result<(), CliError> {
    let args = Args::parse();
    let mut engine = setup::build_engine(&args)?;
    if let Some(path) = &args.eval {
        eval::evaluate(&engine, path, &mut std::io::stdout().lock())?;
        return Ok(());
    }
    run::run(&mut engine, &args, &mut std::io::stdout().lock())?;
    Ok(())
}
