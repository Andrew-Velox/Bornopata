//! An enumerator of exactly one display attribute.

use core::cell::Cell;

use windows::Win32::Foundation::S_FALSE;
use windows::Win32::UI::TextServices::{
    IEnumTfDisplayAttributeInfo, IEnumTfDisplayAttributeInfo_Impl, ITfDisplayAttributeInfo,
};
use windows::core::{Error, Result, implement};

use super::info;

#[implement(IEnumTfDisplayAttributeInfo)]
pub(super) struct AttributeEnum {
    /// The only attribute has been handed out.
    pub(super) done: Cell<bool>,
}

impl IEnumTfDisplayAttributeInfo_Impl for AttributeEnum_Impl {
    fn Clone(&self) -> Result<IEnumTfDisplayAttributeInfo> {
        Ok(AttributeEnum {
            done: Cell::new(self.done.get()),
        }
        .into())
    }

    fn Next(
        &self,
        count: u32,
        infos: *mut Option<ITfDisplayAttributeInfo>,
        fetched: *mut u32,
    ) -> Result<()> {
        let mut taken = 0u32;
        if count >= 1 && !self.done.get() {
            unsafe { *infos = Some(info()) };
            self.done.set(true);
            taken = 1;
        }
        if !fetched.is_null() {
            unsafe { *fetched = taken };
        }
        // The enumerator rule: S_OK when all asked for were returned, S_FALSE when fewer.
        if taken == count {
            Ok(())
        } else {
            Err(Error::from(S_FALSE))
        }
    }

    fn Reset(&self) -> Result<()> {
        self.done.set(false);
        Ok(())
    }

    fn Skip(&self, count: u32) -> Result<()> {
        if count >= 1 {
            self.done.set(true);
        }
        Ok(())
    }
}
