//! Key handling: what each key does while typing Bangla.

use borno_core::Typed;
use borno_platform::Config;
use borno_platform::protocol::{KeyEvent, KeyOutcome, Mode};

use super::Session;

const BACKSPACE: u32 = 0x08;
const TAB: u32 = 0x09;
const ENTER: u32 = 0x0d;
const ESCAPE: u32 = 0x1b;
const PAGE_UP: u32 = 0x21;
const HOME: u32 = 0x24;
const LEFT: u32 = 0x25;
const UP: u32 = 0x26;
const RIGHT: u32 = 0x27;
const DOWN: u32 = 0x28;
const INSERT: u32 = 0x2d;
const DELETE: u32 = 0x2e;

/// X11 keysyms of the modifier and lock keys (Shift_L to Hyper_R, Caps_Lock, Num_Lock): pressing one alone
/// must neither commit the word nor be eaten.
const MODIFIER_KEYS: std::ops::RangeInclusive<u32> = 0xffe1..=0xffee;
const NUM_LOCK: u32 = 0xff7f;
/// The mapped `VK_SHIFT`, which the plugin sends for the Shift keys.
const SHIFT: u32 = 0x10;

/// Stands for "this key is not a function key"; no real key has this code.
const NO_FUNCTION_KEY: u32 = 0;

/// Next-word suggestions offered after a space.
const SUGGESTIONS: usize = 5;

/// A reply to one key press.
pub struct KeyReply {
    pub outcome: KeyOutcome,

    /// Text to commit before the application sees the key (if it is passed through).
    pub commit: Option<String>,
}

impl KeyReply {
    fn consumed(commit: Option<String>) -> Self {
        Self {
            outcome: KeyOutcome::Consumed,
            commit,
        }
    }

    fn passthrough(commit: Option<String>) -> Self {
        Self {
            outcome: KeyOutcome::Passthrough,
            commit,
        }
    }
}

impl Session {
    /// Handles a key press. Releases are the Router's business.
    pub fn press(&mut self, event: KeyEvent, config: &Config) -> KeyReply {
        let reply = self.dispatch(event, config);
        self.consumed_press = (reply.outcome == KeyOutcome::Consumed).then_some(event.virtual_key);
        reply
    }

    fn dispatch(&mut self, event: KeyEvent, config: &Config) -> KeyReply {
        if self.blocked() {
            return KeyReply::passthrough(None);
        }
        if config.shortcut.toggle.matches(&event) {
            let commit = self.commit_word();
            self.mode = self.mode.toggled();
            return KeyReply::consumed(commit);
        }
        if self.mode == Mode::English
            || MODIFIER_KEYS.contains(&event.virtual_key)
            || event.virtual_key == NUM_LOCK
            || event.virtual_key == SHIFT
        {
            return KeyReply::passthrough(None);
        }
        let composing = !self.engine.composition().is_empty();
        // Printable keys use their ASCII code as the virtual key, which collides with the function key codes
        // ('.' is 0x2e, Delete's code); only a key without a character is a function key.
        let function = if event.character.is_none() {
            event.virtual_key
        } else {
            NO_FUNCTION_KEY
        };
        // Only while there is a window to pin; it comes before the command-key rule because it needs Ctrl and Alt.
        if config.shortcut.pin.matches(&event) && (composing || !self.suggestions.is_empty()) {
            self.pin_toggled = true;
            return KeyReply::consumed(None);
        }
        if event.modifiers.has_command_key() {
            return KeyReply::passthrough(self.commit_word());
        }
        if config.shortcut.forget.matches(&event) {
            return self.forget(composing);
        }
        if !composing && !self.suggestions.is_empty() {
            match function {
                TAB => return self.accept_suggestion(),
                // Down and Up choose among the suggestions, so the arrow neither dismisses them nor reaches the
                // application (whose cursor would move under the window).
                UP | DOWN => {
                    let count = self.suggestions.len();
                    self.suggestion_highlight = if function == DOWN {
                        (self.suggestion_highlight + 1) % count
                    } else {
                        (self.suggestion_highlight + count - 1) % count
                    };
                    return KeyReply::consumed(None);
                }
                ESCAPE => {
                    self.suggestions.clear();
                    return KeyReply::consumed(None);
                }
                _ => {
                    self.suggestions.clear();
                    self.suggestion_highlight = 0;
                }
            }
        }
        match function {
            BACKSPACE => {
                return if composing {
                    self.engine.backspace();
                    KeyReply::consumed(None)
                } else {
                    KeyReply::passthrough(None)
                };
            }
            ENTER => return KeyReply::passthrough(self.commit_word()),
            ESCAPE => {
                return if composing {
                    self.engine.reset();
                    KeyReply::consumed(None)
                } else {
                    KeyReply::passthrough(None)
                };
            }
            UP | DOWN | TAB if composing => {
                let backwards = function == UP || (function == TAB && event.modifiers.shift);
                self.engine.move_highlight(if backwards { -1 } else { 1 });
                return KeyReply::consumed(None);
            }
            PAGE_UP..=HOME | LEFT | RIGHT | INSERT | DELETE => {
                return KeyReply::passthrough(self.commit_word());
            }
            _ => {}
        }
        let Some(ch) = event.character else {
            return KeyReply::passthrough(self.commit_word());
        };
        if composing && ('1'..='9').contains(&ch) {
            let index = ch as usize - '1' as usize;
            if index < self.engine.candidates().len() {
                self.engine.highlight(index);
                return KeyReply::consumed(self.commit_word());
            }
        }
        if ch.is_ascii_digit() && !config.general.bangla_digits {
            return KeyReply::passthrough(self.commit_word());
        }
        match self.engine.type_char(ch) {
            Typed::Composing => KeyReply::consumed(None),
            Typed::Commit(text) if !composing && text == ch.to_string() => {
                KeyReply::passthrough(None)
            }
            Typed::Commit(text) => {
                if composing && ch == ' ' {
                    self.suggestions = self.engine.next_words(SUGGESTIONS);
                    self.suggestion_highlight = 0;
                }
                KeyReply::consumed(Some(text))
            }
        }
    }

    /// The forget shortcut: takes back what was learned about the highlighted candidate or suggestion. While a word or
    /// suggestions are shown the key is ours even when there is nothing to forget, so Shift+Delete does not also cut
    /// text in the application.
    fn forget(&mut self, composing: bool) -> KeyReply {
        if composing {
            self.forgot |= self.engine.forget_highlighted();
            return KeyReply::consumed(None);
        }
        let Some(word) = self.suggestions.get(self.suggestion_highlight).cloned() else {
            return KeyReply::passthrough(None);
        };
        self.forgot |= self.engine.forget_word(&word);
        self.suggestions.remove(self.suggestion_highlight);
        self.suggestion_highlight = self
            .suggestion_highlight
            .min(self.suggestions.len().saturating_sub(1));
        KeyReply::consumed(None)
    }

    /// Tab on a next-word suggestion: commit it and offer what follows.
    fn accept_suggestion(&mut self) -> KeyReply {
        let Some(word) = self.suggestions.get(self.suggestion_highlight).cloned() else {
            return KeyReply::passthrough(None);
        };
        let text = self.engine.accept_next_word(&word);
        self.suggestions = self.engine.next_words(SUGGESTIONS);
        self.suggestion_highlight = 0;
        KeyReply::consumed(Some(text))
    }

    /// A click on candidate `index` of the frame being shown.
    pub fn click(&mut self, index: usize) -> Option<String> {
        if self.engine.composition().is_empty() {
            let word = self.suggestions.get(index).cloned()?;
            let text = self.engine.accept_next_word(&word);
            self.suggestions = self.engine.next_words(SUGGESTIONS);
            self.suggestion_highlight = 0;
            return Some(text);
        }
        if index >= self.engine.candidates().len() {
            return None;
        }
        self.engine.highlight(index);
        self.commit_word()
    }
}
