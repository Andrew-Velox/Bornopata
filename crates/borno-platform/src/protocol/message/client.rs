//! Shell to Server.

use serde::{Deserialize, Serialize};

use super::Event;
use crate::protocol::SessionId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Starts a session. Answered with [`super::ServerMessage::Opened`].
    Open {
        session: SessionId,

        /// The program owning the input context, for logs.
        app: String,

        protocol: u32,

        /// The shell's connection generation, echoed in every [`crate::protocol::Identity`].
        generation: u64,

        /// The shell's identifier of the input context, echoed in every identity.
        context: String,

        /// The token the shell attached its popup channel with, when this context can show the popup surface.
        /// The Server then draws this session's frames there.
        #[serde(default)]
        popup: Option<u64>,
    },

    /// The first message of a popup channel instead of a session connection. Answered with
    /// [`super::ServerMessage::Attached`]; the connection then carries [`crate::protocol::popup`] messages only.
    Attach {
        protocol: u32,
        token: u64,
    },

    /// Ends a session. No reply.
    Close {
        session: SessionId,
    },

    Event {
        session: SessionId,
        event: Event,
    },
}
