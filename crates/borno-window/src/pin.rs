//! Where the user dropped the candidate window: a position kept in memory and in one small file, shared by the thread
//! that drags the window and the Router that decides which window to show.

use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

/// The pinned top-left corner of the card, in the units the platform places windows in: logical pixels of the whole
/// desktop on Wayland, physical pixels of the virtual screen on Windows.
pub type Corner = (f32, f32);

/// The pinned position, if the user has dragged the window somewhere. Share it with an `Arc`.
#[derive(Debug)]
pub struct Pin {
    path: Option<PathBuf>,
    corner: Mutex<Option<Corner>>,
}

impl Pin {
    /// Reads `path`. A missing or unreadable file means nothing is pinned.
    pub fn load(path: PathBuf) -> Self {
        let corner = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| parse(&text));
        Self {
            path: Some(path),
            corner: Mutex::new(corner),
        }
    }

    /// A pin that is never written to disk, for tests and for platforms without a data directory.
    pub fn in_memory() -> Self {
        Self {
            path: None,
            corner: Mutex::new(None),
        }
    }

    pub fn get(&self) -> Option<Corner> {
        *self.corner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Moves the pin without touching the disk, for every step of a drag.
    pub fn move_to(&self, corner: Corner) {
        *self.corner.lock().unwrap_or_else(|e| e.into_inner()) = Some(corner);
    }

    /// Writes the pin to its file, once a drag is over.
    pub fn save(&self) -> io::Result<()> {
        let (Some(path), Some((x, y))) = (&self.path, self.get()) else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, format!("x = {x}\ny = {y}\n"))
    }

    /// Lets the window follow the text cursor again.
    pub fn clear(&self) -> io::Result<()> {
        *self.corner.lock().unwrap_or_else(|e| e.into_inner()) = None;
        match &self.path {
            Some(path) => match std::fs::remove_file(path) {
                Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
                _ => Ok(()),
            },
            None => Ok(()),
        }
    }
}

fn parse(text: &str) -> Option<Corner> {
    let (mut x, mut y) = (None, None);
    for line in text.lines() {
        let (key, value) = line.split_once('=')?;
        let value: f32 = value.trim().parse().ok().filter(|v: &f32| v.is_finite())?;
        match key.trim() {
            "x" => x = Some(value),
            "y" => y = Some(value),
            _ => {}
        }
    }
    Some((x?, y?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("borno-pin-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("window-position")
    }

    #[test]
    fn a_saved_position_comes_back_after_a_restart() {
        let path = temp("saved");
        let pin = Pin::load(path.clone());
        assert_eq!(pin.get(), None);
        pin.move_to((120.5, 640.0));
        pin.save().unwrap();
        assert_eq!(Pin::load(path.clone()).get(), Some((120.5, 640.0)));
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn a_position_moved_but_not_saved_is_not_on_disk() {
        let path = temp("unsaved");
        let pin = Pin::load(path.clone());
        pin.move_to((1.0, 2.0));
        assert_eq!(pin.get(), Some((1.0, 2.0)));
        assert_eq!(Pin::load(path).get(), None);
    }

    #[test]
    fn clearing_forgets_the_position_and_the_file() {
        let path = temp("cleared");
        let pin = Pin::load(path.clone());
        pin.move_to((5.0, 6.0));
        pin.save().unwrap();
        pin.clear().unwrap();
        assert_eq!(pin.get(), None);
        assert!(!path.exists());
        pin.clear().unwrap();
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn a_damaged_file_means_nothing_is_pinned() {
        for text in [
            "",
            "x = 1\n",
            "x = a\ny = 2\n",
            "garbage",
            "x = inf\ny = 1\n",
        ] {
            assert_eq!(parse(text), None, "{text:?}");
        }
        assert_eq!(parse("x = -4\ny = 9.5\n"), Some((-4.0, 9.5)));
    }
}
