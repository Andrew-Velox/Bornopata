//! Starts the Linux Server: config, logging, shared engine parts, then the socket loop.

use std::path::Path;
use std::sync::Arc;

use borno_gloss::Glossary;
use borno_learning::FrequencyLearner;
use borno_linux_server::popup::PopupSink;
use borno_linux_server::shared::FilePhrases;
use borno_linux_server::{Router, ServerError, Shared, ipc};
use borno_lm::BigramModel;
use borno_platform::{Config, paths};

/// What a new `phrases.tsv` starts with.
const PHRASE_TEMPLATE: &str = include_str!("phrases-template.tsv");

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
    let glossary = load_glossary(&config);
    write_phrase_template(&user_dir);
    let shared = Shared::new(Arc::from(words), learner, model)
        .with_glossary(glossary)
        .with_shortcuts(config.general.shortcuts)
        .with_emoji(config.general.emoji)
        .with_phrases(FilePhrases::open(paths::phrases_path()));
    let panel = borno_linux_server::panel::start(&config.panel);
    let popup = borno_linux_server::popup::start(&config.panel);
    let mut router = Router::new(config, shared)
        .with_panel(panel)
        .with_popup(popup.clone().map(|hub| Box::new(hub) as Box<dyn PopupSink>));
    router.watch_config(config_path);
    router.watch_forget_marker(paths::forget_marker_path());

    extern "C" fn stop(_: libc::c_int) {
        ipc::request_shutdown();
    }
    unsafe {
        libc::signal(libc::SIGTERM, stop as *const () as libc::sighandler_t);
        libc::signal(libc::SIGINT, stop as *const () as libc::sighandler_t);
    }
    let socket = ipc::socket_path();
    tracing::info!(path = %socket.display(), "Bornopata Linux Server started");
    ipc::serve_with_popup(socket, &mut router, popup)?;
    let reload = ipc::reload_requested();
    // Dropping the Router writes the learned data, so nothing is lost when the process is replaced.
    drop(router);
    if reload {
        return Err(restart().into());
    }
    Ok(())
}

/// Starts over with the new settings: the same process becomes a fresh Server (same pid, so systemd sees nothing),
/// and a shell reconnects on its next key. Only returns, with the reason, when that did not work.
fn restart() -> std::io::Error {
    use std::os::unix::process::CommandExt;
    tracing::info!("starting over with the changed config");
    // The first argument is the path the Server was started with: unlike /proc/self/exe it still names the right file
    // when an upgrade has replaced the program while it was running.
    let program = std::env::args_os()
        .next()
        .map(std::path::PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::current_exe().ok());
    match program {
        Some(program) => std::process::Command::new(program)
            .args(std::env::args_os().skip(1))
            .exec(),
        None => std::io::Error::other("cannot tell which program to start again"),
    }
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

/// The English glosses beside the candidates, when they are turned on and the file is there. A file that cannot be read
/// is logged and the candidates simply have no gloss.
fn load_glossary(config: &Config) -> Option<Glossary> {
    if !config.general.gloss {
        tracing::info!("glosses are turned off in the config");
        return None;
    }
    let Some(path) = paths::gloss_path() else {
        tracing::info!("no gloss file, candidates have no gloss");
        return None;
    };
    match Glossary::open(&path) {
        Ok(glossary) => {
            tracing::info!(path = %path.display(), entries = glossary.len(), "using the glosses");
            Some(glossary)
        }
        Err(error) => {
            tracing::warn!(%error, path = %path.display(), "the gloss file cannot be used");
            None
        }
    }
}

/// The phrase file with its instructions, so the user finds it where the Server looks. Never overwrites.
fn write_phrase_template(user_dir: &Path) {
    let path = user_dir.join(paths::PHRASES_FILE);
    if path.exists() {
        return;
    }
    if let Err(error) =
        std::fs::create_dir_all(user_dir).and_then(|()| std::fs::write(&path, PHRASE_TEMPLATE))
    {
        tracing::warn!(%error, path = %path.display(), "cannot write the phrase template");
    }
}

fn load_model(user_dir: &Path) -> Result<Option<BigramModel>, ServerError> {
    let path = user_dir.join(LANGUAGE_MODEL_FILE);
    if !path.is_file() {
        return Ok(None);
    }
    Ok(Some(BigramModel::from_path(&path)?))
}
