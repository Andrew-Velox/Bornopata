//! Starts the Linux Server: config, logging, shared engine parts, then the socket loop.

use borno_linux_server::popup::PopupSink;
use borno_linux_server::{Router, ServerError, ipc};
use borno_platform::paths;
use borno_server::startup::{build_shared, load_config, start_logging};

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
    let (config_path, config) = load_config()?;

    // Keep the guard alive: it flushes the log writer on exit.
    let _guard = start_logging(&config)?;

    let shared = build_shared(&config)?;
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
