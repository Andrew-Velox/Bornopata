//! What the text service, the edit sessions and the composition sink share: the state of the word being typed. TSF runs
//! it all on one apartment thread, so `Rc` and cells are enough.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use windows::Win32::UI::TextServices::{ITfComposition, ITfContext};

pub(crate) struct Shared {
    /// The composition in the document now; it lives across keys.
    composition: RefCell<Option<ITfComposition>>,

    /// A word is being typed (the Server's last frame was not empty); decides how the keys are treated.
    composing: Cell<bool>,

    /// The document of the last key. Losing the focus or deactivating comes without a document, and the word is settled
    /// into this one.
    last_context: RefCell<Option<ITfContext>>,

    /// The application ended the composition itself: the word is plain text already, but the Server still holds it and
    /// has to forget it before the next key.
    server_stale: Cell<bool>,

    /// This thread has the keyboard focus.
    foreground: Cell<bool>,
}

impl Shared {
    pub(crate) fn new() -> Rc<Self> {
        Rc::new(Self {
            composition: RefCell::new(None),
            composing: Cell::new(false),
            last_context: RefCell::new(None),
            server_stale: Cell::new(false),
            foreground: Cell::new(false),
        })
    }

    pub(crate) fn foreground(&self) -> bool {
        self.foreground.get()
    }

    pub(crate) fn set_foreground(&self, value: bool) {
        self.foreground.set(value);
    }

    pub(crate) fn last_context(&self) -> Option<ITfContext> {
        self.last_context.borrow().clone()
    }

    pub(crate) fn set_last_context(&self, context: Option<ITfContext>) {
        *self.last_context.borrow_mut() = context;
    }

    pub(crate) fn take_server_stale(&self) -> bool {
        self.server_stale.replace(false)
    }

    pub(crate) fn composing(&self) -> bool {
        self.composing.get()
    }

    pub(crate) fn set_composing(&self, value: bool) {
        self.composing.set(value);
    }

    pub(crate) fn has_composition(&self) -> bool {
        self.composition.borrow().is_some()
    }

    /// A clone, so no `borrow()` stays alive across a `match` whose arms borrow mutably again.
    pub(super) fn composition(&self) -> Option<ITfComposition> {
        self.composition.borrow().clone()
    }

    pub(super) fn set_composition(&self, composition: Option<ITfComposition>) {
        *self.composition.borrow_mut() = composition;
    }

    pub(super) fn take_composition(&self) -> Option<ITfComposition> {
        self.composition.borrow_mut().take()
    }

    /// The word is over (the application ended it, the connection broke, focus settled it).
    pub(crate) fn end_composing(&self) {
        self.composing.set(false);
    }

    /// Clears all local state about the word without touching the document.
    pub(crate) fn reset(&self) {
        self.set_composition(None);
        self.set_last_context(None);
        self.end_composing();
    }

    /// The application ended the composition: clear it here and remember that the Server still holds the word.
    pub(super) fn terminated(&self) {
        self.reset();
        self.server_stale.set(true);
    }
}
