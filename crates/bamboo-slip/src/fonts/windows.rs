//! Font file lists for Windows: the fonts every installation ships in the Fonts folder. Nirmala UI covers Bengali
//! (Vrinda is the older one), Segoe UI is the interface font and Segoe UI Emoji draws the emoji.

use std::path::PathBuf;

/// `%WINDIR%\Fonts`, or `C:\Windows\Fonts` when the variable is missing.
fn fonts_dir() -> PathBuf {
    let windows =
        std::env::var_os("WINDIR").map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from);
    windows.join("Fonts")
}

fn in_fonts(names: &[&str]) -> Vec<PathBuf> {
    let dir = fonts_dir();
    names.iter().map(|name| dir.join(name)).collect()
}

pub(super) fn ui_fonts() -> Vec<PathBuf> {
    in_fonts(&["segoeui.ttf", "arial.ttf", "tahoma.ttf"])
}

pub(super) fn ui_weight_fonts() -> Vec<PathBuf> {
    in_fonts(&["segoeuib.ttf"])
}

/// Bengali first. Windows 11 ships Nirmala UI as a collection, Windows 10 as separate files; missing ones are skipped.
pub(super) fn script_fonts(_locale: &str) -> Vec<PathBuf> {
    in_fonts(&[
        "Nirmala.ttc",
        "Nirmala.ttf",
        "NirmalaB.ttf",
        "vrinda.ttf",
        "vrindab.ttf",
    ])
}

pub(super) fn emoji_fonts() -> Vec<PathBuf> {
    in_fonts(&["seguiemj.ttf"])
}
