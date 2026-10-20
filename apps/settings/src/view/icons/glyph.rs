//! The sidebar's line icons, drawn on a 16-unit grid with a 1.5-unit round-capped stroke, scaled to `size`.

use bamboo_slip::{Canvas, Color, Icon};

use super::pen::Pen;

#[derive(Debug, Clone, Copy)]
pub enum Glyph {
    Keyboard,
    Command,
    Appearance,
    Phrases,
    Learning,
    About,
}

impl Icon for Glyph {
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32, size: f32, color: Color) {
        let mut pen = Pen {
            x,
            y,
            unit: size / 16.0,
            strokes: Vec::new(),
            fills: Vec::new(),
        };
        match self {
            // A keyboard
            Glyph::Keyboard => {
                pen.round_rect(1.25, 3.75, 13.5, 8.5, 1.75, false);
                for column in [4.0, 6.67, 9.33, 12.0] {
                    pen.circle(column, 6.5, 0.7, true);
                }
                pen.line(&[(5.0, 9.5), (11.0, 9.5)]);
            }
            // The command sign: four loops joined by lines
            Glyph::Command => {
                for (x, y) in [(4.25, 4.25), (11.75, 4.25), (4.25, 11.75), (11.75, 11.75)] {
                    pen.circle(x, y, 2.0, false);
                }
                pen.line(&[(6.25, 4.25), (6.25, 11.75)]);
                pen.line(&[(9.75, 4.25), (9.75, 11.75)]);
                pen.line(&[(4.25, 6.25), (11.75, 6.25)]);
                pen.line(&[(4.25, 9.75), (11.75, 9.75)]);
            }
            // A circle with one half filled
            Glyph::Appearance => {
                pen.circle(8.0, 8.0, 6.5, false);
                let k = 6.5 * 0.552;
                pen.fill_curve(
                    (8.0, 1.5),
                    &[
                        [(8.0 + k, 1.5), (14.5, 8.0 - k), (14.5, 8.0)],
                        [(14.5, 8.0 + k), (8.0 + k, 14.5), (8.0, 14.5)],
                    ],
                );
            }
            // A speech bubble with two lines
            Glyph::Phrases => {
                pen.line(&[
                    (4.0, 13.5),
                    (4.0, 11.5),
                    (3.0, 11.5),
                    (1.75, 10.25),
                    (1.75, 3.75),
                    (3.0, 2.5),
                    (13.0, 2.5),
                    (14.25, 3.75),
                    (14.25, 10.25),
                    (13.0, 11.5),
                    (7.0, 11.5),
                    (4.0, 13.5),
                ]);
                pen.line(&[(4.75, 5.75), (11.25, 5.75)]);
                pen.line(&[(4.75, 8.25), (9.0, 8.25)]);
            }
            // A sprout: what it learns grows
            Glyph::Learning => {
                pen.line(&[(8.0, 14.0), (8.0, 7.5)]);
                pen.curve(
                    (8.0, 8.5),
                    &[
                        [(8.0, 5.0), (5.5, 3.0), (2.5, 3.0)],
                        [(2.5, 6.5), (4.5, 8.5), (8.0, 8.5)],
                    ],
                );
                pen.curve(
                    (8.0, 7.0),
                    &[
                        [(8.0, 4.0), (10.5, 2.0), (13.5, 2.0)],
                        [(13.5, 5.5), (11.5, 7.0), (8.0, 7.0)],
                    ],
                );
            }
            // An i in a circle
            Glyph::About => {
                pen.circle(8.0, 8.0, 6.5, false);
                pen.circle(8.0, 5.0, 0.9, true);
                pen.line(&[(8.0, 7.5), (8.0, 11.25)]);
            }
        }
        let width = 1.5 * pen.unit;
        for path in &pen.strokes {
            canvas.stroke_outline(path, width, color);
        }
        for path in &pen.fills {
            canvas.fill_path(path, color, bamboo_slip::tiny_skia::BlendMode::SourceOver);
        }
    }
}
