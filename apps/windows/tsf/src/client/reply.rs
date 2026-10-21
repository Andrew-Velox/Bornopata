use borno_platform::protocol::{Frame, KeyOutcome};

/// What the Server answered to one event.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    /// Whether the application may see the key.
    pub outcome: KeyOutcome,

    /// Text to put into the document before anything else.
    pub commit: Option<String>,

    /// What to show now: the word being typed (inline) and the candidates (in the Server's window).
    pub frame: Frame,
}
