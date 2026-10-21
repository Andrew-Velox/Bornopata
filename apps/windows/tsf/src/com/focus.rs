//! Thread focus notifications: when this thread gets or loses the keyboard focus the service is told. Switching windows
//! does not call `ITfKeyEventSink::OnSetFocus`; this does.

use windows::Win32::UI::TextServices::{
    ITfSource, ITfThreadFocusSink, ITfThreadFocusSink_Impl, ITfThreadMgr,
};
use windows::core::{Interface, Result, implement};

#[implement(ITfThreadFocusSink)]
struct ThreadFocusSink;

impl ITfThreadFocusSink_Impl for ThreadFocusSink_Impl {
    fn OnSetThreadFocus(&self) -> Result<()> {
        super::service::on_thread_focus(true);
        Ok(())
    }

    fn OnKillThreadFocus(&self) -> Result<()> {
        super::service::on_thread_focus(false);
        Ok(())
    }
}

/// Attaches the sink; returns what [`unadvise`] needs.
pub(super) fn advise(thread_mgr: &ITfThreadMgr) -> Result<(ITfSource, u32)> {
    let source: ITfSource = thread_mgr.cast()?;
    let sink: ITfThreadFocusSink = ThreadFocusSink.into();
    let cookie = unsafe { source.AdviseSink(&ITfThreadFocusSink::IID, &sink)? };
    Ok((source, cookie))
}

pub(super) fn unadvise(source: &ITfSource, cookie: u32) {
    let _ = unsafe { source.UnadviseSink(cookie) };
}
