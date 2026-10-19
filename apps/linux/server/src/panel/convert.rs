//! Protocol frame to renderer frame.

use borno_panel::Rect;
use borno_panel::compositor::CursorRect as WindowCursor;
use borno_platform::protocol::{CursorRect, Frame, Mode, Source};
use borno_render::{Frame as RenderFrame, Mode as RenderMode, Preedit, Row, Source as RowSource};

/// The key that takes a next-word suggestion, shown on the window.
const NEXT_WORD_HINT: &str = "Tab";

pub fn render_rect(cursor: CursorRect) -> WindowCursor {
    WindowCursor {
        rect: Rect::new(cursor.x, cursor.y, cursor.width, cursor.height),
        surface_relative: cursor.surface_relative,
    }
}

/// What the renderer draws for `frame`: the typed letters on top, one row per candidate marked by its source, the
/// highlight, and the next-word key hint. Next-word rows have no number because digits do not pick them.
pub fn render_frame(frame: &Frame) -> RenderFrame {
    let rows: Vec<Row> = frame
        .candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let source = match candidate.source {
                Source::Learned => RowSource::Learned,
                Source::Direct | Source::Dictionary => RowSource::Typed,
                Source::Completion => RowSource::Completion,
                Source::Next => RowSource::Next,
            };
            let mut row = Row::plain(index, candidate.text.clone());
            if source == RowSource::Next {
                row.index.clear();
            }
            row.source = source;
            row
        })
        .collect();
    RenderFrame {
        preedit: (!frame.roman.is_empty())
            .then(|| Preedit::plain(&frame.roman, frame.roman.chars().count())),
        highlighted: (!rows.is_empty()).then(|| frame.highlight.min(rows.len() - 1)),
        rows,
        hint: frame.suggesting.then(|| NEXT_WORD_HINT.to_owned()),
        mode: RenderMode {
            english: frame.mode == Mode::English,
        },
    }
}

#[cfg(test)]
mod tests {
    use borno_platform::protocol::FrameCandidate;

    use super::*;

    fn candidate(text: &str, source: Source) -> FrameCandidate {
        FrameCandidate {
            text: text.to_owned(),
            source,
        }
    }

    #[test]
    fn rows_carry_their_source_and_the_highlight() {
        let frame = Frame {
            roman: "bangl".to_owned(),
            preedit: "বাংল".to_owned(),
            candidates: vec![
                candidate("বাংল", Source::Direct),
                candidate("বাংলা", Source::Learned),
                candidate("বাংলাদেশ", Source::Completion),
            ],
            highlight: 1,
            ..Frame::default()
        };
        let drawn = render_frame(&frame);
        assert_eq!(drawn.preedit.as_ref().unwrap().text(), "bangl");
        assert_eq!(drawn.preedit.as_ref().unwrap().cursor, 5);
        let sources: Vec<_> = drawn.rows.iter().map(|row| row.source).collect();
        assert_eq!(
            sources,
            [RowSource::Typed, RowSource::Learned, RowSource::Completion]
        );
        assert_eq!(drawn.rows[2].index, "3");
        assert_eq!(drawn.highlighted, Some(1));
        assert_eq!(drawn.hint, None);
    }

    #[test]
    fn next_word_suggestions_have_the_hint_and_no_numbers() {
        let frame = Frame {
            candidates: vec![candidate("ভাত", Source::Next), candidate("যাই", Source::Next)],
            suggesting: true,
            ..Frame::default()
        };
        let drawn = render_frame(&frame);
        assert!(drawn.preedit.is_none());
        assert_eq!(drawn.hint.as_deref(), Some("Tab"));
        assert!(drawn.rows.iter().all(|row| row.index.is_empty()));
        assert!(drawn.rows.iter().all(|row| row.source == RowSource::Next));
    }

    #[test]
    fn an_empty_frame_stays_empty_and_a_stray_highlight_is_clamped() {
        assert!(render_frame(&Frame::default()).is_empty());
        let frame = Frame {
            candidates: vec![candidate("আমি", Source::Direct)],
            highlight: 9,
            ..Frame::default()
        };
        assert_eq!(render_frame(&frame).highlighted, Some(0));
    }

    #[test]
    fn the_cursor_rectangle_keeps_its_frame_of_reference() {
        let rect = render_rect(CursorRect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
            surface_relative: true,
        });
        assert_eq!(rect.rect, Rect::new(1.0, 2.0, 3.0, 4.0));
        assert!(rect.surface_relative);
    }
}
