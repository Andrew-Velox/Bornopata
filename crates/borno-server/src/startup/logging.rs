//! The Server's rolling log file: seven days, one file per day, never the typed text.

use borno_platform::{Config, paths};

use crate::error::ServerError;

/// Starts logging to `server.<date>.log` in the log directory. Keep the guard alive: it flushes the writer on exit.
pub fn start_logging(
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
