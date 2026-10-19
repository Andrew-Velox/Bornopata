//! Errors while starting the popup render thread.

#[derive(Debug, thiserror::Error)]
pub enum PopupError {
    #[error("fonts or theme: {0}")]
    Render(String),

    #[error("could not start the render thread: {0}")]
    Thread(#[from] std::io::Error),
}
