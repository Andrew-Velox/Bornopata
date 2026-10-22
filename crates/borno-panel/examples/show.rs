//! Shows a sample frame at a screen position for a few seconds, to check drawing and placement on a real compositor:
//! `cargo run -p borno-panel --example show -- [THEME] [X] [Y] [SECONDS]`. The position is the text cursor in global
//! coordinates. The window can be dragged with the mouse meanwhile; where it is dropped is kept in memory only.

use std::sync::Arc;
use std::time::Duration;

use borno_panel::compositor::CursorRect;
use borno_panel::{Panel, PanelConfig, Pin, Rect};
use borno_render::sample::{bangl, shaping};

fn main() {
    let mut args = std::env::args().skip(1);
    let theme = args.next().unwrap_or_else(|| "bornopata".to_owned());
    let number =
        |text: Option<String>, default: f32| text.and_then(|t| t.parse().ok()).unwrap_or(default);
    let (x, y) = (number(args.next(), 600.0), number(args.next(), 400.0));
    let seconds = number(args.next(), 4.0);
    let config = PanelConfig {
        theme,
        ..PanelConfig::default()
    };
    let panel = match Panel::start(config, Arc::new(Pin::in_memory())) {
        Ok(panel) => panel,
        Err(error) => {
            eprintln!("cannot start the panel: {error}");
            std::process::exit(1);
        }
    };
    let cursor = CursorRect {
        rect: Rect::new(x, y, 2.0, 20.0),
        surface_relative: false,
    };
    panel.show(shaping(), cursor).expect("the panel is running");
    std::thread::sleep(Duration::from_secs_f32(seconds / 2.0));
    panel.show(bangl(), cursor).expect("the panel is running");
    std::thread::sleep(Duration::from_secs_f32(seconds / 2.0));
    panel.hide().expect("the panel is running");
    std::thread::sleep(Duration::from_millis(200));
}
