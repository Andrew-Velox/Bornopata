//! Shell to Server on the popup channel.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopupClientMessage {
    /// The compositor is done with the buffer, or the shell dropped it unshown: the Server may draw into it again.
    Released { buffer: u64 },

    /// The preferred scale of the popup surface, in 120ths (120 is 1.0, 180 is 1.5). Sent once the shell knows it
    /// and whenever it changes.
    Scale { scale_120: u32 },
}
