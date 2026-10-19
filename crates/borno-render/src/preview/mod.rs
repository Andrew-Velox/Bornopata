//! Theme previews: draws the sample frames with a theme directory and writes the pictures into its `preview/`.

mod error;
mod shot;

use std::path::{Path, PathBuf};

pub use error::{PreviewError, ShotError};
pub use shot::Shot;

use crate::sample::{bangl, next_words, shaping};
use crate::{FontLibrary, Frame, Layout, Renderer, Theme};

/// Twice the point size, as sharp as a HiDPI screenshot.
const SCALE: f32 = 2.0;

/// Draws the vertical samples of `theme_dir` (and a dark one unless the theme locks its appearance) into
/// `out_dir`, overwriting files of the same name. A picture that fails is recorded in its [`Shot`] and the rest are
/// still drawn; only a theme that cannot be read or a font library that cannot be built fails the whole run.
/// `family_files` finds font files of a system family by name, as the shell does when it loads theme fonts.
pub fn write_previews(
    theme_dir: &Path,
    out_dir: &Path,
    family_files: impl FnMut(&str) -> Vec<PathBuf>,
) -> Result<Vec<Shot>, PreviewError> {
    let theme = Theme::from_dir(theme_dir, false)?;
    let mut renderer = Renderer::new(FontLibrary::system("en")?);
    renderer.load_theme_fonts(&theme, family_files);
    std::fs::create_dir_all(out_dir).map_err(|source| PreviewError::OutDir {
        path: out_dir.to_owned(),
        source,
    })?;
    let mut shots = vec![
        ("vertical.png", shaping(), Layout::Vertical, false),
        ("completion.png", bangl(), Layout::Vertical, false),
        ("next-words.png", next_words(), Layout::Vertical, false),
    ];
    if theme.locked_dark().is_none() {
        shots.push(("vertical-dark.png", shaping(), Layout::Vertical, true));
    }
    Ok(shots
        .into_iter()
        .map(|(file, frame, layout, dark)| Shot {
            file,
            result: draw(
                &mut renderer,
                &theme.with_dark(dark),
                &frame,
                layout,
                &out_dir.join(file),
            ),
        })
        .collect())
}

fn draw(
    renderer: &mut Renderer,
    theme: &Theme,
    frame: &Frame,
    layout: Layout,
    path: &Path,
) -> Result<PathBuf, ShotError> {
    // Each picture stands alone: no transition from the previous one.
    renderer.forget();
    let rendered = renderer.render(frame, layout, theme, SCALE)?;
    rendered
        .pixmap
        .save_png(path)
        .map_err(|source| ShotError::Save {
            path: path.to_owned(),
            message: source.to_string(),
        })?;
    Ok(path.to_owned())
}
