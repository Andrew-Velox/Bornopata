//! Where a session's frame is drawn: the input-method popup, the layer-shell window, or by the shell itself.

use borno_platform::protocol::{CursorRect, Frame, SessionId};

use super::Router;
use crate::panel::render_frame;

/// A surface of the Server that can show a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Shown {
    /// The layer-shell window, placed from the cursor rectangle.
    Window,

    /// The popup surface of the shell that attached its popup channel with this token.
    Popup(u64),
}

impl Router {
    /// Draws `frame` for session `id` and hides the surface when there is nothing to show. Returns whether a surface
    /// of the Server shows the frame; when it does not (no surface, the cursor is not known and the shell has no
    /// popup, the surface died) the shell has to draw the candidates itself.
    pub(super) fn present(&mut self, id: SessionId, frame: &Frame) -> bool {
        let Some(session) = self.sessions.get(&id) else {
            return false;
        };
        let context = session.context().to_owned();
        let target = if frame.is_empty() {
            None
        } else {
            self.target(session.popup, session.cursor)
        };
        if let Some((shown_id, shown)) = self.visible
            && target != Some(shown)
            && (shown_id == id || target.is_some())
        {
            self.hide_surface(shown);
            self.visible = None;
        }
        match target {
            Some(Shown::Popup(token)) => {
                let shown = self
                    .popup
                    .as_mut()
                    .is_some_and(|popup| popup.show(token, &context, render_frame(frame)));
                if shown {
                    self.visible = Some((id, Shown::Popup(token)));
                } else {
                    tracing::warn!("the popup channel stopped, the Fcitx5 panel takes over");
                }
                shown
            }
            Some(Shown::Window) => {
                let cursor = self.sessions[&id].cursor.expect("a window needs a cursor");
                let shown = self
                    .panel
                    .as_mut()
                    .is_some_and(|panel| panel.show(render_frame(frame), cursor));
                if shown {
                    self.visible = Some((id, Shown::Window));
                } else {
                    tracing::warn!(
                        "the self-drawn candidate window stopped, the Fcitx5 panel takes over"
                    );
                    self.panel = None;
                    self.visible = None;
                }
                shown
            }
            None => false,
        }
    }

    /// Hides what the closed session showed.
    pub(super) fn hide_session(&mut self, id: SessionId) {
        if let Some((shown_id, shown)) = self.visible
            && shown_id == id
        {
            self.hide_surface(shown);
            self.visible = None;
        }
    }

    /// The popup when the shell has one attached, else the window when the cursor is known.
    fn target(&self, popup: Option<u64>, cursor: Option<CursorRect>) -> Option<Shown> {
        if let (Some(token), Some(sink)) = (popup, self.popup.as_ref())
            && sink.attached(token)
        {
            return Some(Shown::Popup(token));
        }
        (self.panel.is_some() && cursor.is_some_and(|rect| rect.is_known()))
            .then_some(Shown::Window)
    }

    fn hide_surface(&mut self, shown: Shown) {
        match shown {
            Shown::Window => {
                if let Some(panel) = self.panel.as_mut() {
                    panel.hide();
                }
            }
            Shown::Popup(token) => {
                if let Some(popup) = self.popup.as_mut() {
                    popup.hide(token);
                }
            }
        }
    }
}
