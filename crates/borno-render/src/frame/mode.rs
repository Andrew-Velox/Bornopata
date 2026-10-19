//! Typing mode: which script the keys produce. What to show for it is up to the theme (`when: "mode.english"`
//! picks between two text nodes); the renderer only supplies the fact.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mode {
    /// English typing: the input method passes keys through. `false` is Bangla.
    pub english: bool,
}
