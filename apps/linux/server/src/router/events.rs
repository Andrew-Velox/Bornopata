//! What each event of a session does.

use borno_platform::protocol::{Event, Identity, KeyEvent, KeyOutcome, ServerMessage, SessionId};

use super::Router;
use crate::session::Session;

impl Router {
    pub(super) fn event(&mut self, id: SessionId, event: Event) -> ServerMessage {
        let learning = self.config.general.learning;
        let what = describe(&event);
        // A focus or capability change may have taken the popup away on the shell's side, so it is drawn again whatever it
        // shows.
        if matches!(event, Event::Focus { .. } | Event::Capabilities(_)) {
            self.drawn = None;
        }
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
                tracing::debug!(?rect, "cursor");
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
        let was_visible = self.visible.is_some();
        frame.native = self.present(id, &frame);
        // What happened, never the typed text: the kind of event, the shape of the frame and where it was drawn.
        tracing::debug!(
            session = id.0,
            event = %what,
            outcome = ?reply.outcome,
            rows = frame.candidates.len(),
            suggesting = frame.suggesting,
            composing = !frame.roman.is_empty(),
            native = frame.native,
            was_visible,
            visible = ?self.visible.map(|(_, shown)| shown),
            "event"
        );
        ServerMessage::Result {
            session: id,
            outcome: reply.outcome,
            commit: reply.commit,
            identity,
            frame,
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

/// The kind of event for the log. Keys that produce a character are not named, so what is typed stays out of it.
fn describe(event: &Event) -> String {
    match event {
        Event::Capabilities(_) => "capabilities".into(),
        Event::Reset => "reset".into(),
        Event::Key { event, release } => match event.character {
            Some(_) => format!("key(char, release={release})"),
            None => format!("key(code={:#x}, release={release})", event.virtual_key),
        },
        Event::Deactivate { focus_out, .. } => format!("deactivate(focus_out={focus_out})"),
        Event::Focus { focused } => format!("focus({focused})"),
        Event::Cursor(_) => "cursor".into(),
        Event::Candidate { .. } => "candidate".into(),
    }
}
