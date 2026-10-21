//! Windows only: the COM shell of the text service. The DLL entry points are here, the class factory in [`factory`],
//! the text service object in [`service`] and the registry and TSF profile in [`registry`].
//!
//! - [`key`]: virtual key codes to protocol key events.
//! - [`edit`]: edit sessions, which write into the document, and the screen position of the cursor.
//! - [`composition`]: the word being typed, shown inline; [`display_attribute`]: its underline.
//! - [`context`]: the keyboard-disabled switch of a document (password fields); [`focus`]: the thread focus.
#![allow(non_snake_case)] // The exported Dll* entry points are named by the COM convention.

pub(crate) mod composition;
pub(crate) mod context;
pub(crate) mod display_attribute;
pub(crate) mod edit;
pub(crate) mod factory;
pub(crate) mod focus;
pub(crate) mod key;
pub(crate) mod log;
pub(crate) mod registry;
pub(crate) mod service;

use core::ffi::c_void;
use core::ptr;
use core::sync::atomic::{AtomicIsize, AtomicPtr, AtomicU64, Ordering};

use borno_platform::protocol::SessionId;
use windows::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, E_FAIL, HINSTANCE, HMODULE, S_FALSE, S_OK,
};
use windows::Win32::System::Com::IClassFactory;
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::core::{BOOL, GUID, HRESULT, HSTRING, Interface};

/// The CLSID of the text service. The registry, the TSF profile and [`DllGetClassObject`] all know it.
pub(crate) const CLSID_BORNOPATA: GUID = GUID::from_u128(0x1363c4bb_a7d0_4054_ad76_d5a3c0b681ba);

/// [`CLSID_BORNOPATA`] as the registry spells it; the two must change together.
pub(crate) const CLSID_BORNOPATA_STR: &str = "{1363C4BB-A7D0-4054-AD76-D5A3C0B681BA}";

/// The GUID of the language profile.
pub(crate) const GUID_PROFILE: GUID = GUID::from_u128(0x349d7b39_b963_47de_b270_de1e2548fa61);

/// Bengali (Bangladesh).
pub(crate) const LANGID_BN_BD: u16 = 0x0845;

/// What the system lists the input method as.
pub(crate) const SERVICE_DESCRIPTION: &str = "Bornopata";

/// Live COM objects plus the `LockServer` count, which [`DllCanUnloadNow`] goes by.
static DLL_REFERENCES: AtomicIsize = AtomicIsize::new(0);

/// This DLL's module handle, noted at load.
static DLL_MODULE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

/// Tells one connection of this process from another in the Server's replies.
static GENERATION: AtomicU64 = AtomicU64::new(1);

pub(crate) fn lock_module() {
    DLL_REFERENCES.fetch_add(1, Ordering::SeqCst);
}

pub(crate) fn unlock_module() {
    DLL_REFERENCES.fetch_sub(1, Ordering::SeqCst);
}

/// The full path of this DLL on disk, which `InprocServer32` is registered with.
pub(crate) fn module_path() -> windows::core::Result<HSTRING> {
    let module = HMODULE(DLL_MODULE.load(Ordering::SeqCst));
    let mut buffer = [0u16; 260];
    let length = unsafe { GetModuleFileNameW(Some(module), &mut buffer) } as usize;
    if length == 0 || length >= buffer.len() {
        return Err(E_FAIL.into());
    }
    Ok(HSTRING::from_wide(&buffer[..length]))
}

/// This thread's session number: the thread id, which is unique on the whole system. (TSF client ids repeat across
/// processes.)
pub(crate) fn session_id() -> SessionId {
    SessionId(u64::from(unsafe { GetCurrentThreadId() }))
}

/// A number for each connection this process opens: the session's context name and generation in the Server's replies.
pub(crate) fn next_generation() -> u64 {
    GENERATION.fetch_add(1, Ordering::Relaxed)
}

/// The file name of the host program (`notepad.exe`), which the Server logs for each session.
pub(crate) fn host_app_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|exe| {
            exe.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "unknown".to_owned())
}

#[unsafe(no_mangle)]
extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return E_FAIL;
    }
    if unsafe { *rclsid } != CLSID_BORNOPATA {
        return CLASS_E_CLASSNOTAVAILABLE;
    }
    let factory: IClassFactory = factory::ClassFactory.into();
    unsafe { factory.query(riid, ppv) }
}

#[unsafe(no_mangle)]
extern "system" fn DllCanUnloadNow() -> HRESULT {
    if DLL_REFERENCES.load(Ordering::SeqCst) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}

#[unsafe(no_mangle)]
extern "system" fn DllRegisterServer() -> HRESULT {
    match registry::register() {
        Ok(()) => S_OK,
        Err(error) => {
            log::log(&format!("DllRegisterServer failed: {error}"));
            error.code()
        }
    }
}

#[unsafe(no_mangle)]
extern "system" fn DllUnregisterServer() -> HRESULT {
    match registry::unregister() {
        Ok(()) => S_OK,
        Err(error) => error.code(),
    }
}

#[unsafe(no_mangle)]
extern "system" fn DllMain(instance: HINSTANCE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        DLL_MODULE.store(instance.0, Ordering::SeqCst);
        log::log("DllMain: DLL_PROCESS_ATTACH");
    }
    true.into()
}
