//! The self-drawn candidate window on Windows: a layered window that never takes the focus, on a thread of its own.
//!
//! The Router hands frames to a [`CandidateWindow`] through [`borno_server::panel::PanelSink`]; the window renders
//! them with the theme from the config and puts the bitmap on the screen next to the text cursor. Windows may only
//! be used from the thread that made them, so the Router's thread posts commands to the window thread.

mod canvas;
mod class;
mod drag;
mod layered;
mod monitor;
mod thread;

use std::sync::Arc;
use std::sync::mpsc::Sender;

use borno_platform::PanelConfig;
use borno_platform::PanelLayout;
use borno_platform::paths;
use borno_platform::protocol::CursorRect;
use borno_server::panel::PanelSink;
use borno_window::{PanelConfig as WindowConfig, Pin};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_APP};

/// Wakes the window thread to read its commands.
const WM_WAKE: u32 = WM_APP;

/// What the Router asks the window to do.
enum Command {
    Show(Box<borno_render::Frame>, CursorRect),
    Hide,
}

/// The handle the Router draws with. Dropping it ends the window thread.
pub struct CandidateWindow {
    pin: Arc<Pin>,
    sender: Sender<Command>,
    thread_id: u32,
}

/// Starts the window the config asks for. `None` when it cannot start (no usable font or theme), which is logged:
/// the Server then still works, but shows no candidates.
pub fn start(config: &PanelConfig) -> Option<Box<dyn PanelSink>> {
    let window = WindowConfig {
        theme: config.theme.clone(),
        dark: config.dark,
        layout: match config.layout {
            PanelLayout::Vertical => borno_render::Layout::Vertical,
            PanelLayout::Horizontal => borno_render::Layout::Horizontal,
        },
    };
    let pin = Arc::new(Pin::load(paths::window_position_path()));
    match thread::spawn(window, pin.clone()) {
        Ok((sender, thread_id)) => {
            tracing::info!(theme = %config.theme, "candidate window: ready");
            Some(Box::new(CandidateWindow {
                pin,
                sender,
                thread_id,
            }))
        }
        Err(error) => {
            tracing::error!(%error, "the candidate window could not start, no candidates will show");
            None
        }
    }
}

impl CandidateWindow {
    /// `false` once the thread has gone: the command was not delivered.
    fn post(&self, command: Command) -> bool {
        if self.sender.send(command).is_err() {
            return false;
        }
        unsafe { PostThreadMessageW(self.thread_id, WM_WAKE, WPARAM(0), LPARAM(0)) }.is_ok()
    }
}

impl PanelSink for CandidateWindow {
    fn show(&mut self, frame: borno_render::Frame, cursor: CursorRect) -> bool {
        self.post(Command::Show(Box::new(frame), cursor))
    }

    fn hide(&mut self) {
        self.post(Command::Hide);
    }

    fn pinned(&self) -> bool {
        self.pin.get().is_some()
    }

    fn toggle_pin(&mut self) {
        let result = if self.pin.get().is_some() {
            self.pin.clear()
        } else {
            self.pin.move_to(monitor::default_corner());
            self.pin.save()
        };
        if let Err(error) = result {
            tracing::warn!(%error, "could not save where the window is pinned");
        }
    }
}
