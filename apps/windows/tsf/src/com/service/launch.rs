//! Starts the Server when the connection fails.
//!
//! The installer starts the Server at login (from the Startup folder, where Explorer runs it through `ShellExecute`),
//! and if it was closed or crashed since, the text service starts it again: the Server next to the DLL, with
//! `ShellExecuteW` and not `CreateProcess`, which cannot start a program that asks for `uiAccess` (error 740).
//!
//! Two guards keep it from starting many: a cool-down inside the process (a stream of keys tries once), and a named mutex
//! across processes (several applications that find the Server gone at the same moment start one). A missing program, a
//! failed start or a host that is not a normal desktop application is only logged.
//!
//! The host's integrity level matters: the DLL is loaded into the sign-in screen (LogonUI), the UAC prompt and other
//! SYSTEM programs on the secure desktop, and into sandboxed store applications (Low). `ShellExecute` would there either
//! start a Server with SYSTEM rights on the secure desktop or fail. The Server is started from normal (Medium) desktop
//! applications only.
//!
//! While the installer runs nothing is started: it has just stopped the Server to replace its file, and starting it would
//! lock the file again. The installer holds the named mutex [`INSTALLER_MUTEX`].

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{
    CloseHandle, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, GetLastError, HANDLE,
};
use windows::Win32::Security::{
    GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, TOKEN_MANDATORY_LABEL,
    TOKEN_QUERY, TokenIntegrityLevel,
};
use windows::Win32::System::Threading::{
    CreateMutexW, GetCurrentProcess, OpenMutexW, OpenProcessToken, SYNCHRONIZATION_SYNCHRONIZE,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{HSTRING, PCWSTR, w};

use crate::com::log::log;
use crate::com::module_path;

/// The shortest time between two attempts in one process.
const LAUNCH_COOLDOWN: Duration = Duration::from_secs(5);

/// Held while an application is starting the Server, so the others see it and leave it.
const LAUNCH_MUTEX: PCWSTR = w!("Local\\BornopataServerLaunch");

/// Held by the installer while it runs; the same name as in `bornopata.iss`.
const INSTALLER_MUTEX: PCWSTR = w!("Global\\BornopataInstaller");

/// The integrity level of a normal desktop application (not elevated). Lower is a sandbox, higher is an administrator or
/// SYSTEM.
const MEDIUM_INTEGRITY_RID: u32 = 0x2000;

/// The last time an attempt was made; shared by all text services in the process.
static LAST_LAUNCH: Mutex<Option<Instant>> = Mutex::new(None);

/// Starts the Server next to the DLL, unless something forbids it.
pub(super) fn launch_server() {
    if !cooldown_passed() {
        return;
    }
    if !host_is_plain_desktop_app() {
        log("the host is not a normal desktop application, not starting the Server");
        return;
    }
    if installer_running() {
        log("the installer is running, not starting the Server");
        return;
    }
    let Some(exe) = server_exe() else {
        log("cannot find bornopata-server.exe next to the DLL");
        return;
    };
    let Some(_mutex) = launch_mutex() else {
        // Another application is starting it.
        return;
    };
    let file = HSTRING::from(exe.as_os_str());
    let workdir = exe
        .parent()
        .map(|dir| HSTRING::from(dir.as_os_str()))
        .unwrap_or_default();
    // Returns at once: the Server listens on the pipe within a fraction of a second, and waiting here would block the
    // application's UI thread, which makes the system take the input method away.
    let code = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(file.as_ptr()),
            PCWSTR::null(),
            PCWSTR(workdir.as_ptr()),
            SW_SHOWNORMAL,
        )
    };
    // `ShellExecuteW` succeeded when it returns more than 32.
    if (code.0 as isize) > 32 {
        log("asked for the Server to start");
    } else {
        log(&format!(
            "starting the Server failed, code {}",
            code.0 as isize
        ));
    }
}

/// Whether [`LAUNCH_COOLDOWN`] has passed since the last attempt. The time is taken before the attempt, so a failure also
/// waits.
fn cooldown_passed() -> bool {
    let Ok(mut last) = LAST_LAUNCH.lock() else {
        return false;
    };
    if last.is_some_and(|at| at.elapsed() < LAUNCH_COOLDOWN) {
        return false;
    }
    *last = Some(Instant::now());
    true
}

/// Whether the host is a normal desktop application (integrity level Medium): the last sub authority of the SID in the
/// process token's `TokenIntegrityLevel` is the level.
///
/// Anything that cannot be read counts as no: better not to start (the next login does it) than to start a Server with
/// SYSTEM rights on the secure desktop.
fn host_is_plain_desktop_app() -> bool {
    let mut token = HANDLE::default();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }.is_err() {
        return false;
    }
    let mut needed = 0u32;
    // Two steps: the first only asks how many bytes are needed (and always says the buffer is too small, as it should).
    unsafe {
        let _ = GetTokenInformation(token, TokenIntegrityLevel, None, 0, &mut needed);
    }
    let mut buffer = vec![0u8; needed as usize];
    let queried = needed > 0
        && unsafe {
            GetTokenInformation(
                token,
                TokenIntegrityLevel,
                Some(buffer.as_mut_ptr().cast()),
                needed,
                &mut needed,
            )
        }
        .is_ok();
    let _ = unsafe { CloseHandle(token) };
    if !queried {
        return false;
    }
    let label = unsafe {
        buffer
            .as_ptr()
            .cast::<TOKEN_MANDATORY_LABEL>()
            .read_unaligned()
    };
    let sid = label.Label.Sid;
    let count = unsafe { GetSidSubAuthorityCount(sid) };
    if count.is_null() || unsafe { *count } == 0 {
        return false;
    }
    let rid = unsafe { GetSidSubAuthority(sid, u32::from(*count) - 1) };
    !rid.is_null() && unsafe { *rid } == MEDIUM_INTEGRITY_RID
}

/// Whether the installer's mutex exists. An elevated installer makes it, and an ordinary application is refused when it
/// opens it (`ERROR_ACCESS_DENIED`), which says it exists too.
fn installer_running() -> bool {
    match unsafe { OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, false, INSTALLER_MUTEX) } {
        Ok(handle) => {
            let _ = unsafe { CloseHandle(handle) };
            true
        }
        Err(error) => error.code() == ERROR_ACCESS_DENIED.to_hresult(),
    }
}

/// The mutex across processes; `None` when another process holds it (it is starting the Server).
fn launch_mutex() -> Option<LaunchMutex> {
    let handle = unsafe { CreateMutexW(None, false, LAUNCH_MUTEX) }.ok()?;
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        let _ = unsafe { CloseHandle(handle) };
        return None;
    }
    Some(LaunchMutex(handle))
}

/// The handle of the held mutex, closed when dropped.
struct LaunchMutex(HANDLE);

impl Drop for LaunchMutex {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}

fn server_exe() -> Option<PathBuf> {
    let module = module_path().ok()?;
    Some(server_exe_path(Path::new(&module.to_string())))
}

/// The Server sits next to the DLL; the installer puts them in one folder.
fn server_exe_path(module: &Path) -> PathBuf {
    module.with_file_name("bornopata-server.exe")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::server_exe_path;

    #[test]
    fn the_server_sits_next_to_the_dll() {
        assert_eq!(
            server_exe_path(Path::new(
                r"D:\Program Files\Bornopata\bornopata_tsf-0.1.0-dev.dll"
            )),
            Path::new(r"D:\Program Files\Bornopata\bornopata-server.exe")
        );
    }
}
