//! `ITfDisplayAttributeProvider`: hands the underline of the word being typed to the system.

use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::UI::TextServices::{
    IEnumTfDisplayAttributeInfo, ITfDisplayAttributeInfo, ITfDisplayAttributeProvider_Impl,
};
use windows::core::{GUID, Result};

use super::TextService_Impl;
use crate::com::display_attribute;

/// The system looks up the style of the composition (the inline underline) by the display attribute category.
impl ITfDisplayAttributeProvider_Impl for TextService_Impl {
    fn EnumDisplayAttributeInfo(&self) -> Result<IEnumTfDisplayAttributeInfo> {
        Ok(display_attribute::enumerator())
    }

    fn GetDisplayAttributeInfo(&self, guid: *const GUID) -> Result<ITfDisplayAttributeInfo> {
        if unsafe { *guid } == display_attribute::GUID_DISPLAY_ATTRIBUTE_INPUT {
            Ok(display_attribute::info())
        } else {
            Err(E_INVALIDARG.into())
        }
    }
}
