//! Starts the Windows Server: config, logging, shared engine parts, the candidate window, then the pipe loop.
//!
//! A program with no console window: it runs in the background from the moment the user logs in.
#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
fn main() {
    if std::env::args().any(|argument| argument == "--version") {
        // No console to print to; a message box would be noise. The version is in the file properties.
        return;
    }
    if let Err(error) = windows_main::run() {
        // The log is the only place to say it: there is no console.
        tracing::error!(%error, "Bornopata could not start");
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("the Bornopata Windows Server runs on Windows only");
    std::process::exit(1);
}

#[cfg(windows)]
mod windows_main {
    use borno_platform::paths;
    use borno_platform::protocol::DEFAULT_PIPE_NAME;
    use borno_server::ipc::Exit;
    use borno_server::startup::{build_shared, load_config, start_logging};
    use borno_server::{Router, ServerError};
    use borno_windows_server::{pipe, window};

    pub fn run() -> Result<(), ServerError> {
        let (config_path, config) = load_config()?;

        // Keep the guard alive: it flushes the log writer on exit.
        let _guard = start_logging(&config)?;

        let shared = build_shared(&config)?;
        let panel = window::start(&config.panel);
        let mut router = Router::new(config, shared).with_panel(panel);
        router.watch_config(config_path);
        router.watch_forget_marker(paths::forget_marker_path());

        tracing::info!("Bornopata Windows Server started");
        let exit = pipe::serve(DEFAULT_PIPE_NAME, &mut router)?;
        // Dropping the Router writes the learned data, so nothing is lost when the process is replaced.
        drop(router);
        if exit == Exit::Reload {
            restart();
        }
        Ok(())
    }

    /// Starts over with the changed config: a new Server takes over the pipe while this one finishes. The new one waits
    /// a few seconds for the name, see `pipe::serve`.
    fn restart() {
        tracing::info!("starting over with the changed config");
        match std::env::current_exe().and_then(|exe| std::process::Command::new(exe).spawn()) {
            Ok(_) => {}
            Err(error) => tracing::error!(%error, "could not start the Server again"),
        }
    }
}
