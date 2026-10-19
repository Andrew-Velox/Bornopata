//! Which display a reply belongs to.

use serde::{Deserialize, Serialize};

/// Lets the shell drop a reply that arrives for a context or connection that no longer exists, and lets the
/// Server drop a candidate click that refers to a frame that was replaced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    /// The shell's connection generation, constant for one connection to the Server.
    pub generation: u64,

    /// The shell's own identifier of the input context (for Fcitx5, its UUID in hex).
    pub context: String,

    /// Increases with every reply the Server sends for the session.
    pub revision: u64,
}
