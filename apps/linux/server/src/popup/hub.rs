//! The handle the Router and the connections use to reach the render thread.

use std::collections::HashSet;
use std::os::unix::net::UnixStream;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use borno_panel::{PanelConfig as WindowConfig, load_theme};
use borno_platform::{PanelConfig, PanelLayout};
use borno_render::{FontLibrary, Frame, Layout, Renderer};

use super::PopupSink;
use super::error::PopupError;
use super::worker::Worker;

enum Command {
    Attach {
        token: u64,
        stream: UnixStream,
    },
    Detach {
        token: u64,
    },
    Released {
        token: u64,
        buffer: u64,
    },
    Scale {
        token: u64,
        scale_120: u32,
    },
    Show {
        token: u64,
        context: String,
        frame: Box<Frame>,
    },
    Hide {
        token: u64,
    },
}

/// Cloneable handle to the render thread, which ends when the last handle is dropped.
#[derive(Clone)]
pub struct PopupHub {
    commands: Sender<Command>,
    attached: Arc<Mutex<HashSet<u64>>>,
}

impl PopupHub {
    /// Starts the render thread with the theme and layout of `config`.
    pub fn start(config: &PanelConfig) -> Result<Self, PopupError> {
        let window = WindowConfig {
            theme: config.theme.clone(),
            dark: config.dark,
            layout: match config.layout {
                PanelLayout::Vertical => Layout::Vertical,
                PanelLayout::Horizontal => Layout::Horizontal,
            },
        };
        let attached = Arc::new(Mutex::new(HashSet::new()));
        let (commands, receiver) = mpsc::channel::<Command>();
        let (ready, started) = mpsc::channel::<Result<(), PopupError>>();
        let shared = attached.clone();
        // The renderer holds raw pointers and is not `Send`, so it is built on the thread that uses it.
        thread::Builder::new()
            .name("popup-render".to_owned())
            .spawn(move || match build_worker(&window, shared) {
                Ok(mut worker) => {
                    let _ = ready.send(Ok(()));
                    run(&mut worker, receiver);
                }
                Err(error) => {
                    let _ = ready.send(Err(error));
                }
            })?;
        started
            .recv()
            .map_err(|_| PopupError::Render("the render thread stopped".to_owned()))??;
        Ok(Self { commands, attached })
    }

    /// Registers the write side of a popup channel the shell attached with `token`. The Router uses the channel from
    /// now on; it must be called after the shell was told `Attached`.
    pub fn attach(&self, token: u64, stream: UnixStream) {
        let _ = self.commands.send(Command::Attach { token, stream });
        self.attached.lock().unwrap().insert(token);
    }

    pub fn detach(&self, token: u64) {
        let _ = self.commands.send(Command::Detach { token });
    }

    pub fn released(&self, token: u64, buffer: u64) {
        let _ = self.commands.send(Command::Released { token, buffer });
    }

    pub fn scale(&self, token: u64, scale_120: u32) {
        let _ = self.commands.send(Command::Scale { token, scale_120 });
    }
}

impl PopupSink for PopupHub {
    fn show(&mut self, token: u64, context: &str, frame: Frame) -> bool {
        self.attached(token)
            && self
                .commands
                .send(Command::Show {
                    token,
                    context: context.to_owned(),
                    frame: Box::new(frame),
                })
                .is_ok()
    }

    fn hide(&mut self, token: u64) {
        let _ = self.commands.send(Command::Hide { token });
    }

    fn attached(&self, token: u64) -> bool {
        self.attached.lock().unwrap().contains(&token)
    }
}

fn build_worker(
    window: &WindowConfig,
    attached: Arc<Mutex<HashSet<u64>>>,
) -> Result<Worker, PopupError> {
    let theme = load_theme(window).map_err(|error| PopupError::Render(error.to_string()))?;
    let library =
        FontLibrary::system("en").map_err(|error| PopupError::Render(error.to_string()))?;
    let mut renderer = Renderer::new(library);
    renderer.load_theme_fonts(&theme, |_| Vec::new());
    Ok(Worker::new(renderer, theme, window.layout, attached))
}

fn run(worker: &mut Worker, commands: mpsc::Receiver<Command>) {
    loop {
        let received = match worker.next_timeout() {
            Some(wait) => match commands.recv_timeout(wait) {
                Ok(command) => Some(command),
                Err(mpsc::RecvTimeoutError::Timeout) => None,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            },
            None => match commands.recv() {
                Ok(command) => Some(command),
                Err(_) => return,
            },
        };
        match received {
            Some(Command::Attach { token, stream }) => worker.attach(token, stream),
            Some(Command::Detach { token }) => worker.detach(token),
            Some(Command::Released { token, buffer }) => worker.released(token, buffer),
            Some(Command::Scale { token, scale_120 }) => worker.scale(token, scale_120),
            Some(Command::Show {
                token,
                context,
                frame,
            }) => worker.show(token, context, *frame),
            Some(Command::Hide { token }) => worker.hide(token),
            None => {}
        }
        worker.tick_if_due();
    }
}
