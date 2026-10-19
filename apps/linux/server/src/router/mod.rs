//! The input logic of the Server: sessions and what each event does. No sockets here.

mod events;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use borno_platform::Config;
use borno_platform::protocol::{ClientMessage, PROTOCOL_VERSION, ServerMessage, SessionId};

use crate::session::Session;
use crate::shared::Shared;

/// Learned data is written this long after the last change, so typing never waits for the disk.
const FLUSH_AFTER: Duration = Duration::from_secs(3);

/// How often the main loop should call [`Router::tick`].
const TICK: Duration = Duration::from_secs(1);

pub struct Router {
    config: Config,
    shared: Shared,
    sessions: HashMap<SessionId, Session>,

    /// When learned data last changed and has not been written yet.
    unsaved_since: Option<Instant>,
}

impl Router {
    pub fn new(config: Config, shared: Shared) -> Self {
        Self {
            config,
            shared,
            sessions: HashMap::new(),
            unsaved_since: None,
        }
    }

    /// Handles one message from a shell. `None` means there is nothing to answer.
    pub fn handle(&mut self, message: ClientMessage) -> Option<ServerMessage> {
        match message {
            ClientMessage::Open {
                session,
                protocol,
                generation,
                context,
                app,
            } => {
                tracing::info!(session = session.0, app, "session opened");
                if protocol != PROTOCOL_VERSION {
                    return None;
                }
                let created = Session::new(self.shared.engine(), generation, context);
                let mode = created.mode();
                self.sessions.insert(session, created);
                Some(ServerMessage::Opened {
                    session,
                    protocol: PROTOCOL_VERSION,
                    mode,
                })
            }
            ClientMessage::Close { session } => {
                self.sessions.remove(&session);
                None
            }
            ClientMessage::Event { session, event } => Some(self.event(session, event)),
        }
    }

    /// Writes unsaved learned data once the typing has been quiet for a moment.
    pub fn tick(&mut self) {
        if self
            .unsaved_since
            .is_some_and(|since| since.elapsed() >= FLUSH_AFTER)
        {
            self.save();
        }
    }

    /// How long the main loop may sleep before calling [`Self::tick`] again.
    pub fn next_tick(&self) -> Duration {
        TICK
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    fn save(&mut self) {
        if let Err(error) = self.shared.flush() {
            tracing::warn!(%error, "could not save learned data");
        }
        self.unsaved_since = None;
    }
}

impl Drop for Router {
    fn drop(&mut self) {
        if self.shared.learning() {
            self.save();
        }
    }
}
