//! One-off actions behind buttons: open a file, a folder or a web page, and forget what was learned.

use std::path::Path;
use std::process::Command;

use borno_platform::paths;

pub const WEBSITE_URL: &str = "https://andrew-velox.github.io/Bornopata-Web/";

pub const REPOSITORY_URL: &str = "https://github.com/Andrew-Velox/Bornopata";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    OpenConfig,
    OpenPhrasesFile,
    OpenDataDir,
    OpenLogDir,
    OpenWebsite,
    OpenRepository,
    /// The first press only turns the button into "press again to forget everything".
    ForgetEverything,
}

impl Action {
    /// Which row the sentence about the result hangs under.
    pub fn slot(self) -> &'static str {
        match self {
            Self::OpenConfig | Self::OpenPhrasesFile | Self::OpenDataDir | Self::OpenLogDir => {
                "files"
            }
            Self::OpenWebsite | Self::OpenRepository => "links",
            Self::ForgetEverything => "forget",
        }
    }

    /// Does it; the sentence to show afterwards, or `None` when the result is visible (a window opened).
    pub fn run(self, config: &Path) -> Option<String> {
        let result = match self {
            Self::OpenConfig => open(&config.to_string_lossy()),
            Self::OpenPhrasesFile => open_phrases(),
            Self::OpenDataDir => open(&paths::user_dir().to_string_lossy()),
            Self::OpenLogDir => open(&paths::log_dir().to_string_lossy()),
            Self::OpenWebsite => open(WEBSITE_URL),
            Self::OpenRepository => open(REPOSITORY_URL),
            Self::ForgetEverything => forget_everything(),
        };
        match result {
            Ok(message) => (!message.is_empty()).then_some(message),
            Err(error) => {
                tracing::warn!(?self, %error, "an action failed");
                Some(error)
            }
        }
    }
}

/// Opens a file, folder or address with whatever the desktop uses for it.
fn open(target: &str) -> Result<String, String> {
    Command::new("xdg-open")
        .arg(target)
        .spawn()
        .map(|_| String::new())
        .map_err(|error| format!("Could not open it (xdg-open: {error})"))
}

/// The phrase file, created with its instructions when the Server has not yet.
fn open_phrases() -> Result<String, String> {
    let path = paths::phrases_path();
    if !path.exists() {
        std::fs::create_dir_all(paths::user_dir()).map_err(|error| error.to_string())?;
        std::fs::write(&path, "# Your own phrases: a code, a Tab, then the text.\n")
            .map_err(|error| error.to_string())?;
    }
    open(&path.to_string_lossy())
}

/// Asks the Server to forget what it learned by dropping a marker file next to the data; it notices within a second,
/// clears its memory and the files, and removes the marker.
fn forget_everything() -> Result<String, String> {
    std::fs::create_dir_all(paths::user_dir()).map_err(|error| error.to_string())?;
    std::fs::write(paths::forget_marker_path(), b"").map_err(|error| error.to_string())?;
    Ok("Asked the Server to forget everything it learned.".to_owned())
}
