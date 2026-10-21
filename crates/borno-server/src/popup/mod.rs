//! The popup surface of a shell as the Router sees it: a sink for the frames the shell shows itself.

/// Where the Router sends the frame the shell should show in its popup. A fake implements it in tests.
pub trait PopupSink: Send {
    /// Shows `frame` in the popup of channel `token` for input context `context`. `false` when the channel is gone:
    /// the frame was not shown and the shell has to draw it.
    fn show(&mut self, token: u64, context: &str, frame: borno_render::Frame) -> bool;

    fn hide(&mut self, token: u64);

    /// Whether a shell has a popup channel attached with `token`.
    fn attached(&self, token: u64) -> bool;
}
