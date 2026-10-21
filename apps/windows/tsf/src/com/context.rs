//! Switches of a document: TSF requires a keyboard text service to look at `GUID_COMPARTMENT_KEYBOARD_DISABLED` (any
//! value but zero means every key goes to the application and nothing is composed). Password fields are marked this
//! way, and so are empty contexts.

use windows::Win32::UI::TextServices::{
    GUID_COMPARTMENT_EMPTYCONTEXT, GUID_COMPARTMENT_KEYBOARD_DISABLED, ITfCompartmentMgr,
    ITfContext,
};
use windows::core::{GUID, Interface};

/// The document has the keyboard disabled (a password field, an empty context): every key is let through. Not being able
/// to read the switch counts as enabled.
pub(crate) fn keyboard_disabled(context: &ITfContext) -> bool {
    let Ok(manager) = context.cast::<ITfCompartmentMgr>() else {
        return false;
    };
    flag(&manager, &GUID_COMPARTMENT_KEYBOARD_DISABLED)
        || flag(&manager, &GUID_COMPARTMENT_EMPTYCONTEXT)
}

/// The `DWORD` in a compartment of the document is not zero. Not set, or of another type, counts as zero.
fn flag(manager: &ITfCompartmentMgr, guid: &GUID) -> bool {
    unsafe { manager.GetCompartment(guid) }
        .and_then(|compartment| unsafe { compartment.GetValue() })
        .ok()
        .and_then(|variant| i32::try_from(&variant).ok())
        .is_some_and(|value| value != 0)
}
