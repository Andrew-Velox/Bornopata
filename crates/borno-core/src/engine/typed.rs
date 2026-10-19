//! What typing one character did.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Typed {
    /// The character joined the word being composed; read the candidates from the engine.
    Composing,

    /// The character ended the word (or stood alone): this text goes to the application.
    Commit(String),
}
