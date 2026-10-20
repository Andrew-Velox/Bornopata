//! The candidate-window previews: the real renderer draws a theme's sample frame, once for each set of parameters.

use std::collections::HashMap;
use std::sync::Arc;

use borno_render::{FontLibrary, Layout, Pixmap, Renderer, Theme, sample};

/// One preview's parameters: theme id, horizontal, how many rows, scale (bit pattern).
type PreviewKey = (String, bool, usize, u32);

pub struct Previews {
    renderer: Option<Renderer>,

    cache: HashMap<PreviewKey, Arc<Pixmap>>,
}

impl Previews {
    pub fn new() -> Self {
        let renderer = match FontLibrary::system("en") {
            Ok(library) => Some(Renderer::new(library)),
            Err(error) => {
                tracing::warn!(%error, "no fonts for the previews, they are not drawn");
                None
            }
        };
        Self {
            renderer,
            cache: HashMap::new(),
        }
    }

    /// `rows` candidates of the sample frame in `theme`; `None` when it cannot be drawn.
    pub fn get(
        &mut self,
        theme: &Theme,
        horizontal: bool,
        rows: usize,
        scale: f32,
    ) -> Option<Arc<Pixmap>> {
        let key = (theme.id().to_owned(), horizontal, rows, scale.to_bits());
        if let Some(pixmap) = self.cache.get(&key) {
            return Some(Arc::clone(pixmap));
        }
        let renderer = self.renderer.as_mut()?;
        renderer.load_theme_fonts(theme, |_| Vec::new());
        // Every preview starts from nothing, with no transition from the one before.
        renderer.forget();
        let layout = if horizontal {
            Layout::Horizontal
        } else {
            Layout::Vertical
        };
        let mut frame = sample::showcase();
        frame.rows.truncate(rows);
        match renderer.render(&frame, layout, &theme.with_animations(false), scale) {
            Ok(rendered) => {
                let pixmap = Arc::new(rendered.pixmap);
                self.cache.insert(key, Arc::clone(&pixmap));
                Some(pixmap)
            }
            Err(error) => {
                tracing::warn!(%error, theme = theme.id(), "a preview could not be drawn");
                None
            }
        }
    }

    /// Like [`Self::get`], but sized to fit `fit` (width, height in pixels): a bigger one is drawn again at a smaller
    /// scale, so the letters stay sharp. The ratio is cut in steps of 5%, so dragging the window does not draw a new
    /// picture for every pixel of width.
    pub fn fitted(
        &mut self,
        theme: &Theme,
        horizontal: bool,
        rows: usize,
        scale: f32,
        fit: (f32, f32),
    ) -> Option<Arc<Pixmap>> {
        let full = self.get(theme, horizontal, rows, scale)?;
        let ratio = (fit.0 / full.width() as f32).min(fit.1 / full.height() as f32);
        if ratio >= 1.0 {
            return Some(full);
        }
        let ratio = ((ratio * 20.0).floor() / 20.0).max(0.05);
        self.get(theme, horizontal, rows, scale * ratio)
    }
}
