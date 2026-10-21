//! Notices that the config file was changed on disk, so the Server can start over with it.

use std::path::PathBuf;
use std::time::SystemTime;

use borno_platform::Config;

/// What the file looked like when it was last read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Stamp {
    modified: SystemTime,
    len: u64,
}

pub(super) struct ConfigWatch {
    path: PathBuf,
    seen: Option<Stamp>,
}

impl ConfigWatch {
    /// Watches `path`; the file as it is now counts as already seen.
    pub(super) fn new(path: PathBuf) -> Self {
        let seen = stamp(&path);
        Self { path, seen }
    }

    /// Whether the file changed since the last look and now reads as a valid config. A file that does not parse (a
    /// half-typed edit) is reported once and waited out: the Server keeps running with the config it has until the
    /// file is valid again.
    pub(super) fn changed(&mut self) -> bool {
        let now = stamp(&self.path);
        if now == self.seen {
            return false;
        }
        self.seen = now;
        match Config::load(&self.path) {
            Ok(_) => true,
            Err(error) => {
                tracing::warn!(%error, "the config file changed but cannot be used; keeping the old settings");
                false
            }
        }
    }
}

fn stamp(path: &PathBuf) -> Option<Stamp> {
    let meta = std::fs::metadata(path).ok()?;
    Some(Stamp {
        modified: meta.modified().ok()?,
        len: meta.len(),
    })
}
