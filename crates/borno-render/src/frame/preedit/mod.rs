//! The typed letters on the top line: segments plus a caret the renderer draws.

mod segment;
mod style;

pub use segment::PreeditSegment;
pub use style::PreeditStyle;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Preedit {
    /// Segments drawn in order.
    pub segments: Vec<PreeditSegment>,

    /// Caret position in characters of the joined text.
    pub cursor: usize,
}

impl Preedit {
    /// A single plain segment.
    pub fn plain(text: &str, cursor: usize) -> Self {
        Self {
            segments: vec![PreeditSegment {
                text: text.to_owned(),
                style: PreeditStyle::Typed,
            }],
            cursor,
        }
    }

    /// The joined text.
    pub fn text(&self) -> String {
        self.segments.iter().map(|s| s.text.as_str()).collect()
    }

    /// The text before the caret, for measuring where the caret goes.
    pub fn before_cursor(&self) -> String {
        self.text().chars().take(self.cursor).collect()
    }
}
