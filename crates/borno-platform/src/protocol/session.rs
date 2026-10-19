//! Session identity.

use serde::{Deserialize, Serialize};

/// One input context of the shell. The shell numbers them; the Server only uses the number as a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SessionId(pub u64);
