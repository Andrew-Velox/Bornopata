//! Server to shell.

use serde::{Deserialize, Serialize};

use crate::protocol::{Frame, Identity, KeyOutcome, Mode, SessionId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Reply to `Open`.
    Opened {
        session: SessionId,
        protocol: u32,
        mode: Mode,
    },

    /// Reply to an event: what to do with the key, text to commit first, and what to show.
    Result {
        session: SessionId,
        outcome: KeyOutcome,

        /// Text to commit before anything else happens.
        commit: Option<String>,

        identity: Identity,
        frame: Frame,
    },

    /// Reply to `Attach`.
    Attached { protocol: u32 },

    /// The event was for a session the Server does not know (a late callback); nothing to do.
    Ignored { session: SessionId },
}

impl ServerMessage {
    /// The same message addressed to another session number. The connection layer maps the shell's numbers to its own.
    pub fn with_session(mut self, id: SessionId) -> Self {
        match &mut self {
            Self::Opened { session, .. }
            | Self::Result { session, .. }
            | Self::Ignored { session } => {
                *session = id;
            }
            Self::Attached { .. } => {}
        }
        self
    }
}
