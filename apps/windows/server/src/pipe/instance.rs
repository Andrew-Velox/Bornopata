//! One instance of the named pipe: who may connect, creating it and waiting for a client.

use std::fs::File;
use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle};

use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_PIPE_CONNECTED, HANDLE};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
use windows::Win32::Storage::FileSystem::{FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows::core::{HRESULT, HSTRING};

const BUFFER_SIZE: u32 = 64 * 1024;

/// Everyone, ALL APPLICATION PACKAGES and ALL RESTRICTED APPLICATION PACKAGES may connect, at the Low integrity level:
/// with the default access rules the sandboxed applications (the taskbar search, Settings) cannot reach the Server.
const PIPE_SDDL: &str = "D:(A;;GA;;;WD)(A;;GA;;;AC)(A;;GA;;;S-1-15-2-2)S:(ML;;NW;;;LW)";

/// The access rules above. A null descriptor, when they cannot be built, falls back to the default ones.
pub(super) fn security_descriptor() -> PSECURITY_DESCRIPTOR {
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    let converted = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            &HSTRING::from(PIPE_SDDL),
            SDDL_REVISION_1,
            &mut descriptor,
            None,
        )
    };
    if let Err(error) = converted {
        tracing::warn!(%error, "cannot build the pipe access rules, sandboxed applications may not connect");
        return PSECURITY_DESCRIPTOR::default();
    }
    descriptor
}

/// Whether creating the first instance failed because another Server already holds the name.
pub(super) fn is_taken(error: &io::Error) -> bool {
    error.raw_os_error() == Some(ERROR_ACCESS_DENIED.0 as i32)
}

/// Makes one instance. The `File` owns the handle: when the other end closes, std reads it as the end of the stream.
pub(super) fn create_instance(
    name: &str,
    descriptor: PSECURITY_DESCRIPTOR,
    first: bool,
) -> io::Result<File> {
    let attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: false.into(),
    };
    let attributes = (!descriptor.0.is_null()).then_some(&raw const attributes);
    let mut open_mode = PIPE_ACCESS_DUPLEX;
    if first {
        open_mode |= FILE_FLAG_FIRST_PIPE_INSTANCE;
    }
    let handle = unsafe {
        CreateNamedPipeW(
            &HSTRING::from(name),
            open_mode,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            PIPE_UNLIMITED_INSTANCES,
            BUFFER_SIZE,
            BUFFER_SIZE,
            0,
            attributes,
        )
    };
    if handle.is_invalid() {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { File::from_raw_handle(handle.0) })
}

/// Blocks until a client connects to this instance.
pub(super) fn wait_client(stream: File) -> io::Result<File> {
    let handle = HANDLE(stream.as_raw_handle());
    // A client that connected between creating the instance and this call is fine too.
    if let Err(error) = unsafe { ConnectNamedPipe(handle, None) }
        && error.code() != HRESULT::from_win32(ERROR_PIPE_CONNECTED.0)
    {
        return Err(io::Error::other(error));
    }
    Ok(stream)
}
