//! Everything one frame of the candidate window shows: the typed letters on top, the candidate rows, the highlight,
//! a hint on the right of the top line and the typing mode. A display shape only, no ranking or lookup.

mod mode;
mod preedit;
mod row;
mod source;
mod tone;

pub use mode::Mode;
pub use preedit::{Preedit, PreeditSegment, PreeditStyle};
pub use row::Row;
pub use source::Source;
pub use tone::Tone;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Frame {
    /// The typed Roman letters on the top line; `None` when the letters are shown inline only.
    pub preedit: Option<Preedit>,

    pub rows: Vec<Row>,

    /// The highlighted row; `None` for no highlight.
    pub highlighted: Option<usize>,

    /// A short hint on the right of the top line, such as the next-word key.
    pub hint: Option<String>,

    pub mode: Mode,
}

impl Frame {
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty() && self.preedit.is_none() && self.hint.is_none()
    }

    /// Whether the top line is drawn at all.
    pub fn has_top_line(&self) -> bool {
        self.preedit.is_some() || self.hint.is_some()
    }
}
