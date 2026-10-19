//! How a preedit segment is drawn.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreeditStyle {
    /// Typed letters, normal strength.
    Typed,

    /// Letters after the caret, lighter.
    Rest,
}
