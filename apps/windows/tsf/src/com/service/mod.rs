//! The text service object, one for every thread: `ITfTextInputProcessor` (activate and deactivate, [`processor`]),
//! `ITfKeyEventSink` (the keys, [`key_sink`]) and the display attribute provider ([`display`]).
//! The connection to the Server is in [`connection`], writing into the document in [`document`], and starting the Server
//! when it is not running in [`launch`].

mod connection;
mod display;
mod document;
mod key_sink;
mod launch;
mod processor;
pub(crate) mod resend;

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;

use windows::Win32::UI::TextServices::{
    ITfDisplayAttributeProvider, ITfKeyEventSink, ITfSource, ITfTextInputProcessor, ITfThreadMgr,
};
use windows::core::implement;

use super::composition::Shared;
use crate::client::pipe::PipeStream;
use crate::client::{Decision, EngineClient};

/// The session with the Server, shared by the text service and the edit sessions (all on one apartment thread). `None`
/// while there is no connection; keys then go to the application.
pub(crate) type SharedClient = Rc<RefCell<Option<EngineClient<PipeStream>>>>;

/// How long to wait after a failed connection before trying again. Every attempt runs on the UI thread of the
/// application, so it cannot be tried at every key.
const RECONNECT_INTERVAL: Duration = Duration::from_secs(2);

/// One text service, for one thread.
#[implement(ITfTextInputProcessor, ITfKeyEventSink, ITfDisplayAttributeProvider)]
pub struct TextService {
    /// The thread manager from activation; deactivation uses it to undo what was set up.
    thread_mgr: RefCell<Option<ITfThreadMgr>>,

    /// The client id TSF gave this service.
    client_id: Cell<u32>,

    engine: SharedClient,

    /// The word being typed, which lives across keys.
    shared: Rc<Shared>,

    /// When the last attempt to connect failed, to wait [`RECONNECT_INTERVAL`].
    last_connect_failure: Cell<Option<std::time::Instant>>,

    /// The thread focus notification (the source and its cookie); removed at deactivation.
    focus_sink: RefCell<Option<(ITfSource, u32)>>,

    /// What the Server decided for the key `OnTestKeyDown` asked about, for `OnKeyDown` to apply: a key must reach the
    /// Server once only, or a letter would be typed twice.
    pending: RefCell<Option<(u32, Decision)>>,

    /// Keys whose press this service ate; their release is eaten too, so the application never sees a release alone.
    eaten: RefCell<HashSet<u32>>,
}

thread_local! {
    /// The service active on this thread, for the focus notifications. Set by `Activate`, cleared by `Deactivate`.
    static ACTIVE: RefCell<Option<windows::core::ComObject<TextService>>> =
        const { RefCell::new(None) };
}

/// This thread got or lost the keyboard focus (see [`super::focus`]). Losing it settles the word being typed.
pub(super) fn on_thread_focus(foreground: bool) {
    let service = ACTIVE.with(|active| active.borrow().clone());
    if let Some(service) = service {
        service.set_focus(foreground);
    }
}

impl TextService {
    #[allow(clippy::new_without_default)] // It counts itself in the module's references.
    pub fn new() -> Self {
        crate::com::lock_module();
        let engine: SharedClient = Rc::new(RefCell::new(None));
        Self {
            thread_mgr: RefCell::new(None),
            client_id: Cell::new(0),
            engine,
            shared: Shared::new(),
            last_connect_failure: Cell::new(None),
            focus_sink: RefCell::new(None),
            pending: RefCell::new(None),
            eaten: RefCell::new(HashSet::new()),
        }
    }
}

impl Drop for TextService {
    fn drop(&mut self) {
        crate::com::unlock_module();
    }
}
