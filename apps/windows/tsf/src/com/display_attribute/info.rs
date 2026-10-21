//! The one display attribute Bornopata has: the dotted underline of the word being typed. Users cannot change it.

use windows::Win32::UI::TextServices::{
    ITfDisplayAttributeInfo, ITfDisplayAttributeInfo_Impl, TF_DISPLAYATTRIBUTE,
};
use windows::core::{BSTR, GUID, Result, implement};

use super::{GUID_DISPLAY_ATTRIBUTE_INPUT, input_attribute};

#[implement(ITfDisplayAttributeInfo)]
pub(super) struct AttributeInfo;

impl ITfDisplayAttributeInfo_Impl for AttributeInfo_Impl {
    fn GetGUID(&self) -> Result<GUID> {
        Ok(GUID_DISPLAY_ATTRIBUTE_INPUT)
    }

    fn GetDescription(&self) -> Result<BSTR> {
        Ok(BSTR::from("Bornopata input"))
    }

    fn GetAttributeInfo(&self, attribute: *mut TF_DISPLAYATTRIBUTE) -> Result<()> {
        unsafe { *attribute = input_attribute() };
        Ok(())
    }

    fn SetAttributeInfo(&self, _attribute: *const TF_DISPLAYATTRIBUTE) -> Result<()> {
        Ok(())
    }

    fn Reset(&self) -> Result<()> {
        Ok(())
    }
}
