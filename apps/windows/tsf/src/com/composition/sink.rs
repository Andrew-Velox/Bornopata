//! The composition sink: the framework calls it when the application ends our composition by force (a click elsewhere).

use std::rc::Rc;

use windows::Win32::UI::TextServices::{
    ITfComposition, ITfCompositionSink, ITfCompositionSink_Impl,
};
use windows::core::{Ref, Result, implement};

use super::Shared;
use crate::com::log::log;

/// The framework has made the word plain text by then. It only clears the local state and notes that the Server still
/// holds the word; it must never insert what the Server hands over, or the word would be there twice.
#[implement(ITfCompositionSink)]
pub(super) struct CompositionSink {
    shared: Rc<Shared>,
}

impl CompositionSink {
    pub(super) fn new(shared: Rc<Shared>) -> Self {
        Self { shared }
    }
}

impl ITfCompositionSink_Impl for CompositionSink_Impl {
    fn OnCompositionTerminated(
        &self,
        _ecwrite: u32,
        _composition: Ref<ITfComposition>,
    ) -> Result<()> {
        log("the application ended the composition, clearing the local state");
        self.shared.terminated();
        Ok(())
    }
}
