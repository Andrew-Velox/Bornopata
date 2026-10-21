//! The panel thread: a wlr-layer-shell surface that shows what `borno-render` draws.
//!
//! The surface is on the overlay layer, anchored to the top left of the monitor with margins that place it, takes
//! no keyboard focus and has an empty input region, so it never steals focus or clicks from the application.

mod handlers;
mod state;

use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use smithay_client_toolkit::reexports::calloop::channel;

use crate::command::Command;
use crate::compositor::CursorRect;
use crate::error::PanelError;
use borno_render::Frame;
use borno_window::PanelConfig;

/// A running panel. Dropping it stops the thread.
pub struct Panel {
    sender: channel::Sender<Command>,
    thread: Option<JoinHandle<()>>,
}

impl Panel {
    /// Connects to the compositor on a new thread and checks that layer-shell is there. Returns an error when it
    /// is not (GNOME, no Wayland session), so the caller can fall back to another panel.
    pub fn start(config: PanelConfig) -> Result<Self, PanelError> {
        let (sender, receiver) = channel::channel::<Command>();
        let (ready_sender, ready) = mpsc::channel::<Result<(), PanelError>>();
        let thread = thread::Builder::new()
            .name("borno-panel".to_owned())
            .spawn(move || state::run(config, receiver, ready_sender))
            .map_err(|error| PanelError::Protocol(error.to_string()))?;
        match ready.recv() {
            Ok(Ok(())) => Ok(Self {
                sender,
                thread: Some(thread),
            }),
            Ok(Err(error)) => {
                let _ = thread.join();
                Err(error)
            }
            Err(_) => Err(PanelError::Stopped),
        }
    }

    /// Shows `frame` next to the text cursor. Fails when the thread has stopped.
    pub fn show(&self, frame: Frame, cursor: CursorRect) -> Result<(), PanelError> {
        self.sender
            .send(Command::Show {
                frame: Box::new(frame),
                cursor,
            })
            .map_err(|_| PanelError::Stopped)
    }

    pub fn hide(&self) -> Result<(), PanelError> {
        self.sender
            .send(Command::Hide)
            .map_err(|_| PanelError::Stopped)
    }

    /// Whether the thread is still running.
    pub fn is_alive(&self) -> bool {
        self.thread
            .as_ref()
            .is_some_and(|thread| !thread.is_finished())
    }
}

impl Drop for Panel {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
