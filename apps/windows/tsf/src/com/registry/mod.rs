//! Self-registration: writes COM's `InprocServer32` and, through `ITfInputProcessorProfiles` and `ITfCategoryMgr`,
//! registers Bornopata as a keyboard text service. It writes `HKEY_CLASSES_ROOT`, so regsvr32 needs administrator
//! rights. The icon file is handled by [`icon`].

mod icon;

use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoUninitialize,
};
use windows::Win32::UI::TextServices::{
    CLSID_TF_CategoryMgr, CLSID_TF_InputProcessorProfiles, GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
    GUID_TFCAT_TIP_KEYBOARD, GUID_TFCAT_TIPCAP_COMLESS, GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
    GUID_TFCAT_TIPCAP_INPUTMODECOMPARTMENT, GUID_TFCAT_TIPCAP_SECUREMODE,
    GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT, GUID_TFCAT_TIPCAP_UIELEMENTENABLED, ITfCategoryMgr,
    ITfInputProcessorProfiles,
};
use windows::core::{GUID, Result};
use windows_registry::CLASSES_ROOT;

use crate::com::{
    CLSID_BORNOPATA, CLSID_BORNOPATA_STR, GUID_PROFILE, LANGID_BN_BD, SERVICE_DESCRIPTION,
};

/// Besides "keyboard text service" the immersive, system tray and similar abilities are declared, or the input switcher
/// of Windows 10 and 11 leaves the service out (it installs and disappears).
const CATEGORIES: &[GUID] = &[
    GUID_TFCAT_TIP_KEYBOARD,
    GUID_TFCAT_TIPCAP_UIELEMENTENABLED,
    GUID_TFCAT_TIPCAP_SECUREMODE,
    GUID_TFCAT_TIPCAP_COMLESS,
    GUID_TFCAT_TIPCAP_INPUTMODECOMPARTMENT,
    GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
    GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
    GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
];

fn clsid_key() -> String {
    format!("CLSID\\{CLSID_BORNOPATA_STR}")
}

pub(crate) fn register() -> Result<()> {
    let module = crate::com::module_path()?;
    let base = CLASSES_ROOT.create(clsid_key())?;
    base.set_string("", SERVICE_DESCRIPTION)?;
    let inproc = base.create("InprocServer32")?;
    inproc.set_string("", module.to_string())?;
    inproc.set_string("ThreadingModel", "Apartment")?;
    register_profile()
}

/// Undoes [`register`] as far as it can.
pub(crate) fn unregister() -> Result<()> {
    let _ = unregister_profile();
    icon::uninstall();
    let _ = CLASSES_ROOT.remove_tree(clsid_key());
    Ok(())
}

fn register_profile() -> Result<()> {
    com_scope(|| {
        let profiles: ITfInputProcessorProfiles = unsafe {
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?
        };
        // `AddLanguageProfile` reads the strings up to a zero; without one it reads the memory next to them.
        let description = wide_z(SERVICE_DESCRIPTION);
        let icon = icon::install()
            .map(|path| wide_z(&path.to_string_lossy()))
            .unwrap_or_else(|| vec![0]);
        unsafe {
            profiles.Register(&CLSID_BORNOPATA)?;
            profiles.AddLanguageProfile(
                &CLSID_BORNOPATA,
                LANGID_BN_BD,
                &GUID_PROFILE,
                &description,
                &icon,
                0,
            )?;
        }
        let category: ITfCategoryMgr =
            unsafe { CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)? };
        for catid in CATEGORIES {
            unsafe { category.RegisterCategory(&CLSID_BORNOPATA, catid, &CLSID_BORNOPATA)? };
        }
        Ok(())
    })
}

fn unregister_profile() -> Result<()> {
    com_scope(|| {
        if let Ok(category) = unsafe {
            CoCreateInstance::<_, ITfCategoryMgr>(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)
        } {
            for catid in CATEGORIES {
                let _ = unsafe {
                    category.UnregisterCategory(&CLSID_BORNOPATA, catid, &CLSID_BORNOPATA)
                };
            }
        }
        if let Ok(profiles) = unsafe {
            CoCreateInstance::<_, ITfInputProcessorProfiles>(
                &CLSID_TF_InputProcessorProfiles,
                None,
                CLSCTX_INPROC_SERVER,
            )
        } {
            unsafe {
                let _ =
                    profiles.RemoveLanguageProfile(&CLSID_BORNOPATA, LANGID_BN_BD, &GUID_PROFILE);
                let _ = profiles.Unregister(&CLSID_BORNOPATA);
            }
        }
        Ok(())
    })
}

/// regsvr32 has usually set COM up already; when it is in another apartment mode (`RPC_E_CHANGED_MODE`) there is nothing
/// to pair with `CoUninitialize`.
fn com_scope<T>(body: impl FnOnce() -> Result<T>) -> Result<T> {
    let initialized = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.is_ok();
    let result = body();
    if initialized {
        unsafe { CoUninitialize() };
    }
    result
}

/// A zero terminated wide string.
fn wide_z(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
