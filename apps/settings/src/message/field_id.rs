//! The text fields of the program, each with its own editing state.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldId {
    PhraseCode,
    PhraseText,
}
