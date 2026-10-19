//! Where a candidate row came from; themes mark rows by it (`when: "learned"` and so on).

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Source {
    /// A spelling of what was typed.
    #[default]
    Typed,

    /// The user chose this word for this input before.
    Learned,

    /// A word the typed letters are only the beginning of.
    Completion,

    /// A next-word suggestion after a committed word.
    Next,
}
