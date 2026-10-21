//! The icon of the input method: `AddLanguageProfile` wants the path of an icon file. It is compiled into the DLL and
//! written as a machine-wide `.ico` at registration (regsvr32 runs as administrator anyway), which saves a resource
//! compiler in the build.

use std::path::PathBuf;

/// The logo in the sizes Windows asks for.
const ICON: &[u8] = include_bytes!("../../../resources/bornopata.ico");

/// `%ProgramData%\Bornopata\bornopata.ico`. Registration is per machine, so other users must be able to read it: it
/// cannot live in a user's folder.
fn path() -> Option<PathBuf> {
    std::env::var_os("ProgramData")
        .map(|base| PathBuf::from(base).join("Bornopata").join("bornopata.ico"))
}

/// `None` when it cannot be written; the profile is then registered without an icon.
pub(super) fn install() -> Option<PathBuf> {
    let path = path()?;
    let dir = path.parent()?;
    if let Err(error) = std::fs::create_dir_all(dir).and_then(|()| std::fs::write(&path, ICON)) {
        crate::com::log::log(&format!(
            "cannot write the icon, registering without: {error}"
        ));
        return None;
    }
    Some(path)
}

pub(super) fn uninstall() {
    if let Some(path) = path() {
        let _ = std::fs::remove_file(path);
    }
}
