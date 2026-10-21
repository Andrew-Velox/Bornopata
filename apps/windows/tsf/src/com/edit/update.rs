//! The edit session that writes one update: the text to commit and the word being typed.
//!
//! It must be asynchronous (no `TF_ES_SYNC`): the text stores of immersive applications (the new Notepad of Windows 11)
//! are across a process boundary, and synchronous reads and writes make `textinputframework.dll` crash the host. So the
//! callback may run after `OnKeyDown` has returned.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

use windows::Win32::Foundation::E_FAIL;
use windows::Win32::UI::TextServices::{
    ITfContext, ITfEditSession, ITfEditSession_Impl, TF_CONTEXT_EDIT_CONTEXT_FLAGS, TF_ES_READWRITE,
};
use windows::core::{Error, Result, implement};

use crate::com::composition::{Shared, apply};
use crate::com::log::log;
use crate::com::service::SharedClient;

/// A one-shot read and write session that puts one update into `context`.
#[implement(ITfEditSession)]
pub(crate) struct UpdateSession {
    context: ITfContext,

    /// Where the cursor is measured and reported to the Server afterwards.
    engine: SharedClient,

    shared: Rc<Shared>,

    /// The text to settle in the document first.
    commit: Option<String>,

    /// The word being typed; empty ends the composition.
    preedit: String,

    /// A key to send again once the update is written.
    resend: Option<u32>,
}

impl ITfEditSession_Impl for UpdateSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        // Called from the C++ framework: a panic must not cross the FFI boundary.
        let result = catch_unwind(AssertUnwindSafe(|| {
            apply(
                &self.shared,
                &self.engine,
                &self.context,
                ec,
                self.commit.as_deref(),
                &self.preedit,
            )
        }));
        let outcome = match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => {
                log(&format!("the update failed: {error}"));
                Err(error)
            }
            Err(_) => {
                log("the update callback panicked (caught)");
                Err(Error::from(E_FAIL))
            }
        };
        // Also after a failure: the key the application is waiting for must not be lost with the text.
        if let Some(vk) = self.resend {
            crate::com::service::resend::send(vk);
        }
        outcome
    }
}

/// Requests the session. `Ok` only says it was accepted; how it went is logged in the callback.
pub(crate) fn request_update(
    context: &ITfContext,
    client_id: u32,
    engine: SharedClient,
    shared: Rc<Shared>,
    commit: Option<String>,
    preedit: String,
    resend: Option<u32>,
) -> Result<()> {
    let session = UpdateSession {
        context: context.clone(),
        engine,
        shared,
        commit,
        preedit,
        resend,
    };
    request(context, client_id, session.into(), TF_ES_READWRITE)
}

/// Submits an edit session without waiting; the framework keeps the object until the callback has run.
fn request(
    context: &ITfContext,
    client_id: u32,
    session: ITfEditSession,
    flags: TF_CONTEXT_EDIT_CONTEXT_FLAGS,
) -> Result<()> {
    unsafe { context.RequestEditSession(client_id, &session, flags)? }.ok()
}
