//! The user's phrase file, read at start and again whenever it changes on disk.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant, SystemTime};

use borno_core::{PhraseBook, PhraseSource};

/// How often a lookup may look at the file's modification time. A phrase typed seconds after saving works; the typing
/// path does not stat the disk on every key.
const CHECK_EVERY: Duration = Duration::from_secs(2);

/// What the file looked like when it was read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Stamp {
    modified: SystemTime,
    len: u64,
}

struct State {
    book: Arc<PhraseBook>,
    stamp: Option<Stamp>,
    checked: Instant,
}

/// Shared by all engines. A missing file is the same as an empty one.
pub struct FilePhrases {
    path: PathBuf,
    state: Mutex<State>,
}

impl FilePhrases {
    pub fn open(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let (book, stamp) = read(&path);
        Self {
            path,
            state: Mutex::new(State {
                book: Arc::new(book),
                stamp,
                checked: Instant::now(),
            }),
        }
    }

    /// The phrases now, after picking up a change of the file if the last look is old enough.
    pub fn book(&self) -> Arc<PhraseBook> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if state.checked.elapsed() >= CHECK_EVERY {
            state.checked = Instant::now();
            if stamp(&self.path) != state.stamp {
                let (book, stamp) = read(&self.path);
                state.book = Arc::new(book);
                state.stamp = stamp;
            }
        }
        state.book.clone()
    }
}

fn stamp(path: &Path) -> Option<Stamp> {
    let meta = std::fs::metadata(path).ok()?;
    Some(Stamp {
        modified: meta.modified().ok()?,
        len: meta.len(),
    })
}

/// A file that cannot be read gives no phrases; the reason is logged. Bad lines are logged and skipped.
fn read(path: &Path) -> (PhraseBook, Option<Stamp>) {
    let stamp = stamp(path);
    if stamp.is_none() {
        return (PhraseBook::default(), None);
    }
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            tracing::warn!(%error, path = %path.display(), "the phrase file cannot be read");
            return (PhraseBook::default(), stamp);
        }
    };
    let parsed = PhraseBook::parse(&text);
    for problem in &parsed.problems {
        tracing::warn!(path = %path.display(), %problem, "a phrase line is skipped");
    }
    tracing::info!(path = %path.display(), phrases = parsed.book.len(), "using the phrases");
    (parsed.book, stamp)
}

/// One engine's handle on the shared phrases.
pub(super) struct SharedPhrases(pub Arc<FilePhrases>);

impl PhraseSource for SharedPhrases {
    fn lookup(&self, code: &str) -> Vec<String> {
        self.0.book().lookup(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_has_no_phrases() {
        let phrases = FilePhrases::open("/nonexistent/borno/phrases.tsv");
        assert!(phrases.book().is_empty());
    }

    #[test]
    fn a_changed_file_is_read_again() {
        let dir = std::env::temp_dir().join(format!("borno-phrases-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("phrases.tsv");
        std::fs::write(&path, "a\tone\n").unwrap();
        let phrases = FilePhrases::open(&path);
        assert_eq!(phrases.book().lookup("a"), ["one"]);

        std::fs::write(&path, "a\tone\nb\ttwo\n").unwrap();
        phrases.state.lock().unwrap().checked = Instant::now() - CHECK_EVERY;
        assert_eq!(phrases.book().lookup("b"), ["two"]);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
