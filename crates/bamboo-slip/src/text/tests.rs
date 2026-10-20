//! Text shaping checks that need a real Bengali font; they pass quietly where none is installed.

use super::{TextPainter, TextStyle};
use crate::color::Color;
use crate::fonts::FontLibrary;
use crate::text::font::FontSpec;

fn width(painter: &mut TextPainter, text: &str) -> f32 {
    let style = TextStyle::new(FontSpec::new(40.0, 48.0), 40.0, Color::rgb(0, 0, 0), 1.0);
    painter.measure(text, &style).width
}

#[test]
fn a_bengali_conjunct_is_one_narrow_cluster_not_its_letters_side_by_side() {
    let mut painter = TextPainter::new(FontLibrary::system("en").unwrap());
    if !painter.has_family("Noto Sans Bengali") {
        return;
    }
    // প্র is a half-form of প with a ra-phala mark on it. Shaped wrongly it is প, a visible hasanta and a full র.
    let conjunct = width(&mut painter, "প্র");
    let separate = width(&mut painter, "প") + width(&mut painter, "র");
    assert!(conjunct < separate * 0.8, "{conjunct} against {separate}");
    // The same for ন্ত্র, which needs a half-form and a ra-phala together.
    let conjunct = width(&mut painter, "ন্ত্র");
    let separate = width(&mut painter, "ন") + width(&mut painter, "ত") + width(&mut painter, "র");
    assert!(conjunct < separate * 0.8, "{conjunct} against {separate}");
}
