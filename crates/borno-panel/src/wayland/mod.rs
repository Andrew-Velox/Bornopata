//! The panel thread: a wlr-layer-shell surface that shows what `borno-render` draws.
//!
//! The surface is on the overlay layer, anchored to the top left of the monitor with margins that place it, takes
//! no keyboard focus and has an empty input region, so it never steals focus or clicks from the application.

mod handlers;
mod pointer;
mod state;

use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};

use smithay_client_toolkit::reexports::calloop::channel;

use crate::command::Command;
use crate::compositor::{self, CursorRect};
use crate::error::PanelError;
use borno_render::Frame;
use borno_window::{PanelConfig, Pin, Rect};

/// A running panel. Dropping it stops the thread.
pub struct Panel {
    pin: Arc<Pin>,
    sender: channel::Sender<Command>,
    thread: Option<JoinHandle<()>>,
}

impl Panel {
    /// Connects to the compositor on a new thread and checks that layer-shell is there. Returns an error when it
    /// is not (GNOME, no Wayland session), so the caller can fall back to another panel.
    pub fn start(config: PanelConfig, pin: Arc<Pin>) -> Result<Self, PanelError> {
        let (sender, receiver) = channel::channel::<Command>();
        let (ready_sender, ready) = mpsc::channel::<Result<(), PanelError>>();
        let thread = thread::Builder::new()
            .name("borno-panel".to_owned())
            .spawn({
                let pin = pin.clone();
                move || state::run(config, pin, receiver, ready_sender)
            })
            .map_err(|error| PanelError::Protocol(error.to_string()))?;
        match ready.recv() {
            Ok(Ok(())) => Ok(Self {
                pin,
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

    /// Whether the user has pinned the window somewhere.
    pub fn pinned(&self) -> bool {
        self.pin.get().is_some()
    }

    /// Pins the window on the focused monitor, from where the user can drag it, or lets it follow the cursor again.
    pub fn toggle_pin(&self) {
        let result = if self.pin.get().is_some() {
            self.pin.clear()
        } else {
            self.pin.move_to(default_corner());
            self.pin.save()
        };
        if let Err(error) = result {
            tracing::warn!(%error, "could not save where the window is pinned");
        }
    }

    /// Whether the thread is still running.
    pub fn is_alive(&self) -> bool {
        self.thread
            .as_ref()
            .is_some_and(|thread| !thread.is_finished())
    }
}

/// Where the window goes when it is pinned without having been dragged: centred, a little below the middle of the
/// focused monitor, where it is in nobody's way and easy to find.
fn default_corner() -> (f32, f32) {
    let monitor = compositor::detect()
        .and_then(|mut compositor| compositor.focused_monitor())
        .unwrap_or(Rect::new(0.0, 0.0, 1920.0, 1080.0));
    (
        monitor.x + monitor.width * 0.5 - DEFAULT_HALF_WIDTH,
        monitor.y + monitor.height * 0.6,
    )
}

/// About half the width of a vertical window.
const DEFAULT_HALF_WIDTH: f32 = 130.0;

impl Drop for Panel {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
