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
