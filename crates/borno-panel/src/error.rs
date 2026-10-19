//! Why the panel could not start or stopped.

#[derive(Debug, thiserror::Error)]
pub enum PanelError {
    #[error("no Wayland connection: {0}")]
    Connect(String),

    #[error("the compositor does not offer {0}")]
    Missing(&'static str),

    #[error("wayland: {0}")]
    Protocol(String),

    #[error("fonts or theme: {0}")]
    Render(String),

    #[error("the panel thread has stopped")]
    Stopped,
}
