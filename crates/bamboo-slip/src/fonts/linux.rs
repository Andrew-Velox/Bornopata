//! Font file lists for Linux and FreeBSD: the usual places distributions install Noto and other fonts.
//! Debian and Ubuntu use `truetype/` and `opentype/`, Arch uses `noto/` and `noto-cjk/`, Fedora installs variable fonts
//! (`google-noto-vf/`), FreeBSD packages go to `/usr/local/share/fonts`. Reading the fontconfig setup would be the next step.

use std::path::{Path, PathBuf};

pub(super) fn ui_fonts() -> Vec<PathBuf> {
    [
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/google-noto-vf/NotoSans[wght].ttf",
        "/usr/share/fonts/google-noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
        "/usr/local/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/local/share/fonts/dejavu/DejaVuSans.ttf",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}

pub(super) fn ui_weight_fonts() -> Vec<PathBuf> {
    [
        "/usr/share/fonts/truetype/noto/NotoSans-Bold.ttf",
        "/usr/share/fonts/noto/NotoSans-Bold.ttf",
        "/usr/share/fonts/google-noto/NotoSans-Bold.ttf",
        "/usr/local/share/fonts/noto/NotoSans-Bold.ttf",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}

/// Font directories searched for Bengali fonts, one or two levels deep.
const FONT_DIRS: [&str; 4] = [
    "/usr/share/fonts",
    "/usr/local/share/fonts",
    "/usr/share/fonts/truetype",
    "/usr/share/fonts/opentype",
];

/// Name parts of common Bengali fonts, matched case-insensitively against file names.
const BENGALI_NAMES: [&str; 5] = ["bengali", "siliguri", "kalpurush", "solaimanlipi", "mukti"];

/// Script fonts: Bengali first (Noto Sans Bengali, then anything with a Bengali name found on disk), then the
/// CJK fonts the inherited fallback chain still uses.
pub(super) fn script_fonts(_locale: &str) -> Vec<PathBuf> {
    let mut fonts = bengali_fonts();
    fonts.extend(
        [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/google-noto-sans-cjk-vf-fonts/NotoSansCJK-VF.ttc",
            "/usr/share/fonts/google-noto-sans-cjk-fonts/NotoSansCJK-Regular.ttc",
            "/usr/local/share/fonts/noto/NotoSansCJKsc-Regular.otf",
        ]
        .into_iter()
        .map(PathBuf::from),
    );
    fonts
}

/// Bengali font files: the known Noto paths first, then every file under the font directories whose name says
/// Bengali. Only regular and bold weights of Noto Sans Bengali are wanted, so the many other Noto weights and the
/// serif and UI variants do not all get loaded.
pub(super) fn bengali_fonts() -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = [
        "/usr/share/fonts/noto/NotoSansBengali-Regular.ttf",
        "/usr/share/fonts/noto/NotoSansBengali-Bold.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansBengali-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansBengali-Bold.ttf",
        "/usr/share/fonts/google-noto/NotoSansBengali-Regular.ttf",
        "/usr/share/fonts/google-noto/NotoSansBengali-Bold.ttf",
        "/usr/share/fonts/google-noto-vf/NotoSansBengali[wdth,wght].ttf",
    ]
    .into_iter()
    .map(PathBuf::from)
    .filter(|path| path.is_file())
    .collect();
    if found.is_empty() {
        for dir in FONT_DIRS {
            scan(Path::new(dir), 2, &mut found);
        }
        found.sort();
        found.dedup();
        found.truncate(4);
    }
    found
}

fn scan(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if depth > 0 {
                scan(&path, depth - 1, found);
            }
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_lowercase();
        let is_font = name.ends_with(".ttf") || name.ends_with(".otf");
        if is_font && BENGALI_NAMES.iter().any(|part| name.contains(part)) {
            found.push(path);
        }
    }
}

pub(super) fn emoji_fonts() -> Vec<PathBuf> {
    [
        "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
        "/usr/share/fonts/noto/NotoColorEmoji.ttf",
        "/usr/share/fonts/google-noto-color-emoji-fonts/NotoColorEmoji.ttf",
        "/usr/local/share/fonts/noto/NotoColorEmoji.ttf",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}
