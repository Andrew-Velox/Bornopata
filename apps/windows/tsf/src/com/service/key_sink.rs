//! `ITfKeyEventSink`: asks the Server about every key and does what it says.
//!
//! TSF first asks `OnTestKeyDown` whether the input method wants a key and only then sends `OnKeyDown`, so the Server
//! cannot simply be asked in `OnKeyDown`. A key may reach the Server once only (a letter would be typed twice), so the
//! answer to the test is kept and `OnKeyDown` applies it. A key the Server lets through is not eaten and the application
//! handles it. When the Server lets a key through after settling the word (Enter, an arrow, Ctrl+C) the key is eaten and
//! sent again once the text is written, see [`resend`](super::resend).
//!
//! Nothing here logs what is typed, only which kind of key was handled.

use windows::Win32::Foundation::{FALSE, LPARAM, WPARAM};
use windows::Win32::UI::TextServices::{ITfContext, ITfKeyEventSink_Impl};
use windows::core::{BOOL, GUID, Ref, Result};

use super::TextService_Impl;
use super::resend;
use crate::client::Decision;
use crate::com::key::event::{is_modifier_key, to_key_event};
use crate::com::log::log;

impl ITfKeyEventSink_Impl for TextService_Impl {
    /// The document lost or got the focus. Switching windows does not reach this, `focus` handles that; both end in
    /// [`TextService_Impl::set_focus`].
    fn OnSetFocus(&self, foreground: BOOL) -> Result<()> {
        self.set_focus(foreground.as_bool());
        Ok(())
    }

    fn OnTestKeyDown(&self, pic: Ref<ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(self.test_key(&pic, wparam.0 as u32).into())
    }

    fn OnKeyDown(&self, pic: Ref<ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(self.key_down(pic, wparam.0 as u32).into())
    }

    fn OnTestKeyUp(&self, _pic: Ref<ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(self.key_up(wparam.0 as u32).into())
    }

    fn OnKeyUp(&self, _pic: Ref<ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(self.key_up(wparam.0 as u32).into())
    }

    /// Bornopata registers no preserved keys.
    fn OnPreservedKey(&self, _pic: Ref<ITfContext>, _guid: *const GUID) -> Result<BOOL> {
        Ok(FALSE)
    }
}

impl TextService_Impl {
    /// Whether the focus is on this thread: when it leaves, the word being typed is settled into the document.
    pub(super) fn set_focus(&self, foreground: bool) {
        self.shared.set_foreground(foreground);
        self.pending.borrow_mut().take();
        if foreground {
            self.report_focus(true);
        } else {
            self.settle_pending();
            self.report_focus(false);
        }
    }

    /// `OnTestKeyDown`: asks the Server (once per key) and remembers the answer for `OnKeyDown`.
    fn test_key(&self, pic: &Ref<ITfContext>, vk: u32) -> bool {
        if is_modifier_key(vk) || resend::is_ours(vk) {
            return false;
        }
        if let Some((pending, decision)) = self.pending.borrow().as_ref()
            && *pending == vk
        {
            return decision.eat;
        }
        if self.keyboard_disabled(pic) {
            return false;
        }
        let Some(decision) = self.query(pic, vk) else {
            return false;
        };
        let eat = decision.eat;
        // Only a key that is eaten comes back as `OnKeyDown`; one that is let through is the application's.
        if eat {
            *self.pending.borrow_mut() = Some((vk, decision));
        }
        eat
    }

    /// `OnKeyDown`: applies the answer the test got, or asks if it was not asked.
    fn key_down(&self, pic: Ref<ITfContext>, vk: u32) -> bool {
        if is_modifier_key(vk) || resend::is_ours(vk) {
            return false;
        }
        let kept = {
            let mut pending = self.pending.borrow_mut();
            match pending.take() {
                Some((kept_vk, decision)) if kept_vk == vk => Some(decision),
                other => {
                    *pending = other;
                    None
                }
            }
        };
        let decision = match kept {
            Some(decision) => decision,
            None => {
                if self.keyboard_disabled(&pic) {
                    return false;
                }
                match self.query(&pic, vk) {
                    Some(decision) => decision,
                    None => return false,
                }
            }
        };
        if decision.eat {
            self.eaten.borrow_mut().insert(vk);
        }
        if decision.write {
            let resend = decision.resend.then_some(vk);
            self.update_document(pic, decision.commit, decision.preedit, resend);
        }
        decision.eat
    }

    /// A key came up. One this service sent itself passes; one whose press was eaten is eaten, and the Server is told so
    /// that it forgets the press.
    fn key_up(&self, vk: u32) -> bool {
        self.pending.borrow_mut().take();
        if is_modifier_key(vk) {
            return false;
        }
        if resend::is_ours(vk) {
            resend::arrived(vk);
            return false;
        }
        if !self.eaten.borrow_mut().remove(&vk) {
            return false;
        }
        if let Ok(mut guard) = self.engine.try_borrow_mut()
            && let Some(client) = guard.as_mut()
        {
            let _ = client.key(to_key_event(vk), true);
        }
        true
    }

    /// A password field has the keyboard disabled; every key goes to the application and no word is started. A word that
    /// is already being typed is this service's own and carries on.
    fn keyboard_disabled(&self, pic: &Ref<ITfContext>) -> bool {
        if self.shared.composing() {
            return false;
        }
        let Ok(context) = pic.ok() else {
            return false;
        };
        let disabled = crate::com::context::keyboard_disabled(context);
        if disabled {
            log("the document has the keyboard disabled (a password field), letting keys through");
        }
        disabled
    }

    /// Sends the key to the Server and turns the answer into what to do. `None` when there is no connection or the call
    /// failed (the connection is then closed and the key goes to the application).
    fn query(&self, pic: &Ref<ITfContext>, vk: u32) -> Option<Decision> {
        if !self.ensure_connected() {
            return None;
        }
        if let Ok(context) = pic.ok() {
            self.shared.set_last_context(Some(context.clone()));
        }
        let event = to_key_event(vk);
        let mut guard = self.engine.borrow_mut();
        let client = guard.as_mut()?;
        // The application ended our composition earlier: the Server still holds that word, so it forgets it first.
        if self.shared.take_server_stale() {
            let _ = client.reset();
        }
        match client.key(event, false) {
            Ok(reply) => {
                let decision = Decision::from_reply(reply);
                self.shared.set_composing(decision.composing);
                log(&format!(
                    "key {} {} (word being typed: {})",
                    if event.character.is_some() {
                        "char"
                    } else {
                        "function"
                    },
                    if decision.eat { "eaten" } else { "let through" },
                    decision.composing
                ));
                Some(decision)
            }
            Err(error) => {
                log(&format!(
                    "the Server call failed, closing the connection: {error}"
                ));
                *guard = None;
                self.last_connect_failure.set(None);
                self.shared.end_composing();
                None
            }
        }
    }
}
