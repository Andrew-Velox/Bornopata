//! What the framework says about the focused input field.

use serde::{Deserialize, Serialize};

/// Facts from the framework; what to do about them is the Server's decision.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    /// The field asked for no learning from its content (private or sensitive text).
    pub sensitive: bool,

    pub password: bool,

    /// The framework switched input methods off for this field.
    pub disabled: bool,
}
