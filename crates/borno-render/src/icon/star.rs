//! A five-pointed star, drawn filled: marks a word the user chose before.

use bamboo_slip::tiny_skia::{BlendMode, PathBuilder};
use bamboo_slip::{Canvas, Color, Icon};

#[derive(Debug)]
pub(super) struct Star;

impl Icon for Star {
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32, size: f32, color: Color) {
        let (cx, cy) = (x + size / 2.0, y + size / 2.0);
        let (outer, inner) = (size * 0.5, size * 0.21);
        let mut path = PathBuilder::new();
        for point in 0..10 {
            let radius = if point % 2 == 0 { outer } else { inner };
            let angle = std::f32::consts::PI * (point as f32 / 5.0) - std::f32::consts::FRAC_PI_2;
            let (px, py) = (cx + radius * angle.cos(), cy + radius * angle.sin());
            if point == 0 {
                path.move_to(px, py);
            } else {
                path.line_to(px, py);
            }
        }
        path.close();
        if let Some(path) = path.finish() {
            canvas.fill_path(&path, color, BlendMode::SourceOver);
        }
    }
}
