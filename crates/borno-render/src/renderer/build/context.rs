//! The data context while the tree is instantiated: the whole frame plus the row a `repeat` is expanding.
//! Conditions (`when`) and bindings (`bind`) are evaluated here; unknown conditions are false and unknown bindings empty
//! (the node is not drawn). Names are looked up in the row first, then in the frame.

use super::row::RowContext;
use crate::frame::{Frame, Row, Source, Tone};

#[derive(Debug, Clone, Copy)]
pub(super) struct Context<'a> {
    pub(super) frame: &'a Frame,

    /// The row a `repeat` is expanding.
    pub(super) row: Option<RowContext<'a>>,
}

impl<'a> Context<'a> {
    pub(super) fn new(frame: &'a Frame) -> Self {
        Self { frame, row: None }
    }

    /// The context for each row of a list.
    pub(super) fn with_row(self, row: &'a Row, index: usize, count: usize) -> Self {
        Self {
            row: Some(RowContext { row, index, count }),
            ..self
        }
    }

    /// `when` 求值：`a|b` 任一成立，`!a` 取反。
    pub(super) fn holds(&self, condition: &str) -> bool {
        condition.split('|').any(|term| {
            let term = term.trim();
            match term.strip_prefix('!') {
                Some(name) => !self.flag(name),
                None => self.flag(term),
            }
        })
    }

    fn flag(&self, name: &str) -> bool {
        if let Some(RowContext { row, index, count }) = self.row {
            match name {
                "highlighted" => return self.frame.highlighted == Some(index),
                "learned" => return row.source == Source::Learned,
                "completion" => return row.source == Source::Completion,
                "next" => return row.source == Source::Next,
                "annotation" => return !row.annotation.is_empty(),
                "first" => return index == 0,
                "last" => return index + 1 == count,
                _ => {}
            }
        }
        let frame = self.frame;
        match name {
            "preedit" => frame.preedit.is_some(),
            "hint" => frame.hint.is_some(),
            "candidates" => !frame.rows.is_empty(),
            "annotations" => frame.rows.iter().any(|row| !row.annotation.is_empty()),
            "highlighted" => self.highlighted().is_some(),
            "highlighted.annotation" => self
                .highlighted()
                .is_some_and(|row| !row.annotation.is_empty()),
            "suggesting" => frame.rows.iter().any(|row| row.source == Source::Next),
            "mode.english" => frame.mode.english,
            _ => false,
        }
    }

    /// Text bindings.
    pub(super) fn text(&self, name: &str) -> Option<&'a str> {
        if let Some(RowContext { row, .. }) = self.row {
            match name {
                "index" => return Some(&row.index),
                "text" => return Some(&row.text),
                _ => {}
            }
        }
        match name {
            "hint" => self.frame.hint.as_deref(),
            _ => None,
        }
    }

    /// Annotation segment bindings.
    pub(super) fn annotation(&self, name: &str) -> Option<&'a [(String, Tone)]> {
        match (name, self.row) {
            ("annotation", Some(RowContext { row, .. })) => Some(&row.annotation),
            ("highlighted.annotation", _) => {
                self.highlighted().map(|row| row.annotation.as_slice())
            }
            _ => None,
        }
    }

    /// 列表绑定。
    pub(super) fn list(&self, name: &str) -> Option<&'a [Row]> {
        (name == "candidates").then_some(self.frame.rows.as_slice())
    }

    fn highlighted(&self) -> Option<&'a Row> {
        self.frame.highlighted.and_then(|i| self.frame.rows.get(i))
    }
}

#[cfg(test)]
mod tests {
    use super::Context;
    use crate::frame::{Frame, Mode, Row, Source};

    #[test]
    fn frame_and_row_conditions() {
        let mut learned = Row::plain(0, "আমি");
        learned.source = Source::Learned;
        let frame = Frame {
            rows: vec![learned.clone(), Row::plain(1, "অমি")],
            highlighted: Some(0),
            hint: Some("Tab".to_owned()),
            mode: Mode { english: true },
            ..Frame::default()
        };
        let ctx = Context::new(&frame);
        assert!(ctx.holds("mode.english") && !ctx.holds("!mode.english"));
        assert!(ctx.holds("hint") && ctx.holds("candidates") && !ctx.holds("preedit"));
        assert_eq!(ctx.text("hint"), Some("Tab"));
        let first = ctx.with_row(&frame.rows[0], 0, 2);
        assert!(first.holds("learned") && first.holds("highlighted") && first.holds("first"));
        assert!(!first.holds("completion") && !first.holds("last"));
        let second = ctx.with_row(&frame.rows[1], 1, 2);
        assert!(second.holds("last") && !second.holds("learned|highlighted"));
        assert_eq!(second.text("text"), Some("অমি"));
        assert!(!Context::new(&Frame::default()).holds("hint"));
    }
}
