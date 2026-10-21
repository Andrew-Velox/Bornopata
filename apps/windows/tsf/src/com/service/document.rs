//! Writing into the document: the Server's answer to a key goes in through an asynchronous edit session. A word still
//! being typed is settled into the document when the focus leaves or the service is deactivated.

use windows::Win32::UI::TextServices::ITfContext;
use windows::core::Ref;

use super::TextService_Impl;
use crate::com::edit::request_update;
use crate::com::log::log;

impl TextService_Impl {
    /// The focus left or the service goes away: the Server hands over the word being typed and it goes into the document
    /// as it is. If the application has ended the composition itself the word is text already, and the Server only has to
    /// forget it.
    pub(super) fn settle_pending(&self) {
        let stale = self.shared.take_server_stale();
        if !self.shared.composing() && !stale {
            return;
        }
        let commit = {
            let mut guard = self.engine.borrow_mut();
            let Some(client) = guard.as_mut() else {
                self.shared.reset();
                return;
            };
            let reply = if stale {
                client.reset()
            } else {
                client.deactivate()
            };
            match reply {
                Ok(reply) => reply.commit,
                Err(error) => {
                    log(&format!("could not settle the word, closing: {error}"));
                    drop(guard);
                    self.disconnect();
                    return;
                }
            }
        };
        if stale {
            return;
        }
        self.shared.end_composing();
        let Some(context) = self.shared.last_context() else {
            log("no document to settle the word into, dropping it");
            self.shared.reset();
            return;
        };
        let requested = request_update(
            &context,
            self.client_id.get(),
            self.engine.clone(),
            self.shared.clone(),
            commit.filter(|text| !text.is_empty()),
            String::new(),
            None,
        );
        if let Err(error) = requested {
            log(&format!(
                "the edit session to settle the word was refused: {error}"
            ));
            self.shared.reset();
        }
    }

    /// Requests the edit session that writes `commit` and the word being typed into the document of `pic`. `resend` is a
    /// key to send again once it is written.
    pub(super) fn update_document(
        &self,
        pic: Ref<ITfContext>,
        commit: Option<String>,
        preedit: String,
        resend: Option<u32>,
    ) {
        // When the last letter is erased the frame is empty but the composition is still there, and has to be ended.
        if commit.is_none()
            && preedit.is_empty()
            && !self.shared.composing()
            && !self.shared.has_composition()
            && resend.is_none()
        {
            return;
        }
        let Ok(context) = pic.ok() else {
            log("no document, dropping the update");
            return;
        };
        if let Err(error) = request_update(
            context,
            self.client_id.get(),
            self.engine.clone(),
            self.shared.clone(),
            commit,
            preedit,
            resend,
        ) {
            log(&format!("the edit session was refused: {error}"));
        }
    }
}
