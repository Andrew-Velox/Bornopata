//! Events of an open session.

use serde::{Deserialize, Serialize};

use crate::protocol::{Capabilities, CursorRect, Identity, KeyEvent};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    /// The framework's facts about the field. Sent before any other event of the session.
    Capabilities(Capabilities),

    /// The framework reset the context: drop the word being typed.
    Reset,

    Key {
        event: KeyEvent,

        /// A key release. The Server consumes the release of a key whose press it consumed.
        release: bool,
    },

    /// The context lost the input method.
    Deactivate {
        focus_out: bool,

        /// The shell showed the word as client preedit, so the framework commits it on focus out by itself.
        client_preedit: bool,

        capability_changed: bool,
    },

    Focus {
        focused: bool,
    },

    /// The text cursor moved (or the first position of a focused context). Sent before the key that follows it, so
    /// the Server can put its candidate window next to the cursor.
    Cursor(CursorRect),

    /// The user clicked candidate `index` of the frame identified by `identity`.
    Candidate {
        identity: Identity,
        index: usize,
    },
}
