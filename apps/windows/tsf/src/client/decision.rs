//! What to do with a key once the Server has answered: eaten or let through, which text goes into the document, and
//! which word stays inline.

use borno_platform::protocol::KeyOutcome;

use super::Reply;

pub struct Decision {
    /// The key is the input method's: the application must not see it.
    pub eat: bool,

    /// Text to put into the document first.
    pub commit: Option<String>,

    /// The word being typed, shown inline; empty when there is none.
    pub preedit: String,

    /// The document has to be written.
    pub write: bool,

    /// The key must be sent again after the text is in. The Server settled the word and let the key through (Enter, an
    /// arrow, Ctrl+C), but the text can only be written after the key has been answered, and the application has to see
    /// the key after it.
    pub resend: bool,

    /// A word is being typed now.
    pub composing: bool,
}

impl Decision {
    /// The decision for the Server's answer `reply`.
    pub fn from_reply(reply: Reply) -> Self {
        let composing = !reply.frame.roman.is_empty();
        let preedit = match (composing, reply.frame.preedit.is_empty()) {
            (false, _) => String::new(),
            (true, true) => reply.frame.roman.clone(),
            (true, false) => reply.frame.preedit.clone(),
        };
        match (reply.outcome, reply.commit) {
            (KeyOutcome::Consumed, commit) => Self {
                eat: true,
                commit,
                preedit,
                write: true,
                resend: false,
                composing,
            },
            (KeyOutcome::Passthrough, Some(commit)) => Self {
                eat: true,
                commit: Some(commit),
                preedit,
                write: true,
                resend: true,
                composing,
            },
            (KeyOutcome::Passthrough, None) => Self {
                eat: false,
                commit: None,
                preedit,
                write: false,
                resend: false,
                composing,
            },
        }
    }
}
