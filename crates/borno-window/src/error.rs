//! Why a window could not be set up.

#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("theme {theme}: {reason}")]
    Theme { theme: String, reason: String },
}
