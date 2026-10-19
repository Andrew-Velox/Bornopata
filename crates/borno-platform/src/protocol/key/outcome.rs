//! What the Server did with a key.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyOutcome {
    /// The input method took the key; the application must not see it.
    Consumed,

    /// The application gets the key (after any text in the same reply was committed).
    Passthrough,
}
