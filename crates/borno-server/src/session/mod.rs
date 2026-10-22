//! One input context: its engine, mode and what is currently shown.

mod frame;
mod keys;

use borno_core::Engine;
use borno_platform::protocol::{Capabilities, CursorRect, Identity, Mode};

/// The state of one Fcitx5 input context.
pub struct Session {
    pub(crate) engine: Engine,
    pub(crate) mode: Mode,
    pub(crate) capabilities: Capabilities,

    /// The shell's connection generation and context identifier, echoed in every reply.
    generation: u64,
    context: String,

    /// Counts replies; part of the [`Identity`] so a stale candidate click can be recognised.
    revision: u64,

    /// Next-word suggestions offered after a space, shown until the next key.
    pub(crate) suggestions: Vec<String>,
    pub(crate) suggestion_highlight: usize,

    /// Set when a key took something back from the learner, so the Router saves it soon. The Router clears it.
    pub(crate) forgot: bool,

    /// Set when the pin shortcut was pressed, so the Router pins or frees the window. The Router clears it.
    pub(crate) pin_toggled: bool,

    /// Virtual key of the last consumed press. Its release is consumed too, so the application never sees a
    /// release without a press.
    pub(crate) consumed_press: Option<u32>,

    /// Where the text cursor is, as the shell last reported it; the self-drawn window needs it to know where to go.
    pub(crate) cursor: Option<CursorRect>,

    /// The token of the shell's popup channel, when this context can show the input-method popup.
    pub(crate) popup: Option<u64>,
}

impl Session {
    pub fn new(engine: Engine, generation: u64, context: String) -> Self {
        Self {
            engine,
            mode: Mode::Bangla,
            capabilities: Capabilities::default(),
            generation,
            context,
            revision: 0,
            suggestions: Vec::new(),
            suggestion_highlight: 0,
            forgot: false,
            pin_toggled: false,
            consumed_press: None,
            cursor: None,
            popup: None,
        }
    }

    /// The shell's identifier of the input context.
    pub fn context(&self) -> &str {
        &self.context
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Whether the field forbids input method handling altogether.
    pub(crate) fn blocked(&self) -> bool {
        self.capabilities.password || self.capabilities.disabled
    }

    /// The identity for the next reply.
    pub fn next_identity(&mut self) -> Identity {
        self.revision += 1;
        self.identity()
    }

    /// The identity of the last reply.
    pub fn identity(&self) -> Identity {
        Identity {
            generation: self.generation,
            context: self.context.clone(),
            revision: self.revision,
        }
    }

    /// Applies the field's facts: no learning in sensitive fields, nothing at all in password fields.
    pub fn set_capabilities(&mut self, capabilities: Capabilities, learning: bool) {
        let changed = self.capabilities != capabilities;
        self.capabilities = capabilities;
        self.engine
            .set_learning(learning && !capabilities.sensitive);
        // A change of the field's privacy is a boundary: a word typed under the old facts must not leak into the new.
        if changed {
            self.drop_word();
        }
    }

    /// Hides the next-word suggestions.
    pub fn drop_suggestions(&mut self) {
        self.suggestions.clear();
        self.suggestion_highlight = 0;
    }

    /// Forgets the word being typed and the suggestions, committing nothing.
    pub fn drop_word(&mut self) {
        self.engine.reset();
        self.engine.forget_context();
        self.suggestions.clear();
        self.suggestion_highlight = 0;
    }

    /// Commits the word being typed, if any, and clears the suggestions.
    pub fn commit_word(&mut self) -> Option<String> {
        self.suggestions.clear();
        self.suggestion_highlight = 0;
        self.engine.commit()
    }
}
