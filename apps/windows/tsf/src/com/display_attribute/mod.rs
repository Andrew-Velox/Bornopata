//! The underline of the word being typed: a display attribute in the TSF way, "input in progress" on the composition
//! range. The system asks for it through `ITfDisplayAttributeProvider` (implemented by the text service, see
//! `service::display`) and reads [`AttributeInfo`] from the enumerator; when the composition is written the GUID is
//! turned into an atom and set as `GUID_PROP_ATTRIBUTE` on the range.

mod enumerator;
mod info;

use core::cell::Cell;

use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::TextServices::{
    CLSID_TF_CategoryMgr, GUID_PROP_ATTRIBUTE, IEnumTfDisplayAttributeInfo, ITfCategoryMgr,
    ITfContext, ITfDisplayAttributeInfo, ITfRange, TF_ATTR_INPUT, TF_CT_NONE, TF_DA_COLOR,
    TF_DISPLAYATTRIBUTE, TF_LS_DOT,
};
use windows::core::{GUID, Result};

use self::enumerator::AttributeEnum;
use self::info::AttributeInfo;
use super::log::log;

/// Bornopata's own GUID for the composition attribute; it goes with the display attribute category in the registry.
pub(crate) const GUID_DISPLAY_ATTRIBUTE_INPUT: GUID =
    GUID::from_u128(0x131297ab_c4f0_4a41_a2e9_7f923c500f58);

thread_local! {
    /// The GUID's atom in the category manager, worked out when first needed; 0 means not yet.
    static ATOM: Cell<u32> = const { Cell::new(0) };
}

/// A dotted underline; the colors follow the document.
pub(super) fn input_attribute() -> TF_DISPLAYATTRIBUTE {
    let follow_text = TF_DA_COLOR {
        r#type: TF_CT_NONE,
        ..Default::default()
    };
    TF_DISPLAYATTRIBUTE {
        crText: follow_text,
        crBk: follow_text,
        lsStyle: TF_LS_DOT,
        fBoldLine: false.into(),
        crLine: follow_text,
        bAttr: TF_ATTR_INPUT,
    }
}

/// Underlines the composition range. A failure is only logged.
pub(crate) fn mark(context: &ITfContext, ec: u32, range: &ITfRange) {
    if let Err(error) = mark_inner(context, ec, range) {
        log(&format!("could not underline the composition: {error}"));
    }
}

fn mark_inner(context: &ITfContext, ec: u32, range: &ITfRange) -> Result<()> {
    let variant = VARIANT::from(guid_atom()? as i32);
    let property = unsafe { context.GetProperty(&GUID_PROP_ATTRIBUTE)? };
    unsafe { property.SetValue(ec, range, &variant) }
}

fn guid_atom() -> Result<u32> {
    let cached = ATOM.with(Cell::get);
    if cached != 0 {
        return Ok(cached);
    }
    let manager: ITfCategoryMgr =
        unsafe { CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)? };
    let atom = unsafe { manager.RegisterGUID(&GUID_DISPLAY_ATTRIBUTE_INPUT)? };
    ATOM.with(|cell| cell.set(atom));
    Ok(atom)
}

pub(crate) fn enumerator() -> IEnumTfDisplayAttributeInfo {
    AttributeEnum {
        done: Cell::new(false),
    }
    .into()
}

pub(crate) fn info() -> ITfDisplayAttributeInfo {
    AttributeInfo.into()
}
