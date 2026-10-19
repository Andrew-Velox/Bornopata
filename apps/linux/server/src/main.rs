//! Starts the Linux Server: config, logging, shared engine parts, then the socket loop.

use std::path::Path;
use std::sync::Arc;

use borno_learning::FrequencyLearner;
use borno_linux_server::{Router, ServerError, Shared, ipc};
use borno_lm::BigramModel;
use borno_platform::{Config, paths};

/// The optional language model, looked up in the user data directory.
const LANGUAGE_MODEL_FILE: &str = "lm.bnd";

fn main() {
    if std::env::args().any(|argument| argument == "--version") {
        println!("borno-linux-server {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if let Err(error) = run() {
        eprintln!("Bornopata could not start: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ServerError> {
    let config_path = paths::config_path();
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Config::write_template_if_missing(&config_path)?;
    let config = Config::load(&config_path)?;

    // Keep the guard alive: it flushes the log writer on exit.
    let _guard = start_logging(&config)?;

    let user_dir = paths::user_dir();
    let learner = if config.general.learning {
        Some(FrequencyLearner::open(&user_dir)?)
    } else {
        None
    };
    let model = load_model(&user_dir)?;
    let dictionary = paths::dictionary_path();
    match &dictionary {
        Some(path) => tracing::info!(path = %path.display(), "using the dictionary"),
        None => tracing::warn!("no dictionary file, only the scaffold word list answers"),
    }
    let words = borno_dictionary::word_source(dictionary.as_deref())?;
    let shared = Shared::new(Arc::from(words), learner, model);
    let mut router = Router::new(config, shared);

    extern "C" fn stop(_: libc::c_int) {
        ipc::request_shutdown();
    }
    unsafe {
        libc::signal(libc::SIGTERM, stop as *const () as libc::sighandler_t);
        libc::signal(libc::SIGINT, stop as *const () as libc::sighandler_t);
    }
    let socket = ipc::socket_path();
    tracing::info!(path = %socket.display(), "Bornopata Linux Server started");
    ipc::serve(socket, &mut router)?;
    Ok(())
}

fn start_logging(
    config: &Config,
) -> Result<tracing_appender::non_blocking::WorkerGuard, ServerError> {
    std::fs::create_dir_all(paths::log_dir())?;
    let appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("server")
        .filename_suffix("log")
        .max_log_files(7)
        .build(paths::log_dir())
        .map_err(std::io::Error::other)?;
    let (writer, guard) = tracing_appender::non_blocking(appender);
    tracing_subscriber::fmt()
        .with_env_filter(config.general.log_level.directive())
        .with_ansi(false)
        .with_writer(writer)
        .init();
    Ok(guard)
}

fn load_model(user_dir: &Path) -> Result<Option<BigramModel>, ServerError> {
    let path = user_dir.join(LANGUAGE_MODEL_FILE);
    if !path.is_file() {
        return Ok(None);
    }
    Ok(Some(BigramModel::from_path(&path)?))
}
