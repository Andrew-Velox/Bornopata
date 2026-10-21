//! `ITfTextInputProcessor`: activation attaches the key sink and the focus sink and connects to the Server;
//! deactivation undoes it in the opposite order, after the word being typed has been settled.

use windows::Win32::UI::TextServices::{
    ITfKeyEventSink, ITfKeystrokeMgr, ITfTextInputProcessor_Impl, ITfThreadMgr,
};
use windows::core::{IUnknownImpl, Interface, Ref, Result};

use super::{ACTIVE, TextService_Impl};
use crate::com::focus;
use crate::com::log::log;

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, ptim: Ref<ITfThreadMgr>, tid: u32) -> Result<()> {
        let thread_mgr = ptim.ok()?.clone();
        let keystroke: ITfKeystrokeMgr = thread_mgr.cast()?;
        let sink: ITfKeyEventSink = self.to_interface();
        unsafe { keystroke.AdviseKeyEventSink(tid, &sink, true)? };
        self.client_id.set(tid);

        match focus::advise(&thread_mgr) {
            Ok(advice) => *self.focus_sink.borrow_mut() = Some(advice),
            Err(error) => log(&format!("could not listen to the thread focus: {error}")),
        }
        // A thread that already has the focus when activated gets no "got the focus" later.
        let focused = unsafe { thread_mgr.IsThreadFocus() }.is_ok_and(|focus| focus.as_bool());
        *self.thread_mgr.borrow_mut() = Some(thread_mgr);
        self.connect();
        if focused {
            self.set_focus(true);
        }
        ACTIVE.with(|active| *active.borrow_mut() = Some(self.to_object()));
        log(&format!("activated, tid={tid}"));
        Ok(())
    }

    fn Deactivate(&self) -> Result<()> {
        // First the callbacks, so nothing calls into this service afterwards.
        ACTIVE.with(|active| active.borrow_mut().take());
        if let Some((source, cookie)) = self.focus_sink.borrow_mut().take() {
            focus::unadvise(&source, cookie);
        }
        // A word half typed is settled into the document before the session closes.
        self.settle_pending();
        if let Some(thread_mgr) = self.thread_mgr.borrow_mut().take()
            && let Ok(keystroke) = thread_mgr.cast::<ITfKeystrokeMgr>()
        {
            let _ = unsafe { keystroke.UnadviseKeyEventSink(self.client_id.get()) };
        }
        if let Some(client) = self.engine.borrow_mut().take() {
            let _ = client.close();
        }
        self.pending.borrow_mut().take();
        self.eaten.borrow_mut().clear();
        self.shared.reset();
        self.shared.take_server_stale();
        self.shared.set_foreground(false);
        log("deactivated");
        Ok(())
    }
}
