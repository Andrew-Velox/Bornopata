//! What each event of a session does.

use borno_platform::protocol::{
    CursorRect, Event, Frame, Identity, KeyEvent, KeyOutcome, ServerMessage, SessionId,
};

use super::Router;
use crate::panel::render_frame;
use crate::session::Session;

impl Router {
    pub(super) fn event(&mut self, id: SessionId, event: Event) -> ServerMessage {
        let learning = self.config.general.learning;
        let Some(session) = self.sessions.get_mut(&id) else {
            return ServerMessage::Ignored { session: id };
        };
        let reply = match event {
            Event::Capabilities(capabilities) => {
                session.set_capabilities(capabilities, learning);
                Reply::idle()
            }
            Event::Reset => {
                session.drop_word();
                Reply::idle()
            }
            Event::Focus { focused } => {
                if !focused {
                    session.drop_suggestions();
                }
                Reply::idle()
            }
            Event::Deactivate {
                focus_out,
                client_preedit,
                ..
            } => {
                // With client preedit the framework commits the word itself on focus out; committing too would
                // type it twice.
                let commit = if focus_out && client_preedit {
                    None
                } else {
                    session.commit_word()
                };
                session.drop_word();
                Reply {
                    commit,
                    ..Reply::idle()
                }
            }
            Event::Cursor(rect) => {
                session.cursor = Some(rect);
                Reply::idle()
            }
            Event::Key { event, release } => key(session, event, release, &self.config),
            Event::Candidate { identity, index } => {
                if !current(session, &identity) {
                    return ServerMessage::Ignored { session: id };
                }
                match session.click(index) {
                    Some(text) => Reply {
                        outcome: KeyOutcome::Consumed,
                        commit: Some(text),
                    },
                    None => Reply::idle(),
                }
            }
        };
        // Learning happens on commit, so a commit is the moment there is something new to save.
        if reply.commit.is_some() && self.shared.learning() {
            self.unsaved_since
                .get_or_insert_with(std::time::Instant::now);
        }
        let identity = session.next_identity();
        let mut frame = session.frame();
        let cursor = session.cursor;
        frame.native = self.present(id, &frame, cursor);
        ServerMessage::Result {
            session: id,
            outcome: reply.outcome,
            commit: reply.commit,
            identity,
            frame,
        }
    }

    /// Draws `frame` in the self-drawn window next to `cursor`, or hides the window when there is nothing to show.
    /// Returns whether the window shows the frame; when it does not (no window, the cursor is not known yet, the
    /// window died) the shell has to draw the candidates itself.
    fn present(&mut self, id: SessionId, frame: &Frame, cursor: Option<CursorRect>) -> bool {
        let Some(panel) = self.panel.as_mut() else {
            return false;
        };
        if frame.is_empty() {
            if self.visible == Some(id) {
                self.visible = None;
                panel.hide();
            }
            return false;
        }
        let Some(cursor) = cursor else {
            return false;
        };
        if panel.show(render_frame(frame), cursor) {
            self.visible = Some(id);
            true
        } else {
            tracing::warn!("the self-drawn candidate window stopped, the Fcitx5 panel takes over");
            self.panel = None;
            self.visible = None;
            false
        }
    }
}

struct Reply {
    outcome: KeyOutcome,
    commit: Option<String>,
}

impl Reply {
    fn idle() -> Self {
        Self {
            outcome: KeyOutcome::Passthrough,
            commit: None,
        }
    }
}

/// A click is honoured only for the frame the user is looking at.
fn current(session: &Session, identity: &Identity) -> bool {
    *identity == session.identity()
}

fn key(
    session: &mut Session,
    event: KeyEvent,
    release: bool,
    config: &borno_platform::Config,
) -> Reply {
    if release {
        let consumed = session.consumed_press == Some(event.virtual_key);
        if consumed {
            session.consumed_press = None;
        }
        return Reply {
            outcome: if consumed {
                KeyOutcome::Consumed
            } else {
                KeyOutcome::Passthrough
            },
            commit: None,
        };
    }
    let reply = session.press(event, config);
    Reply {
        outcome: reply.outcome,
        commit: reply.commit,
    }
}
