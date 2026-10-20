//! What building one frame needs: the settings, the control theme, the colors and the scale.

use bamboo_slip::WidgetTheme;
use bamboo_slip::taffy::{LengthPercentage, Size};

use super::palette::Palette;
use crate::settings::Settings;

pub struct Page<'a> {
    pub settings: &'a Settings,

    pub theme: WidgetTheme,

    pub palette: &'a Palette,

    pub scale: f32,

    /// The width of the content area minus its side margins (pixels).
    pub content_width: f32,
}

impl Page<'_> {
    pub fn px(&self, points: f32) -> f32 {
        points * self.scale
    }

    pub fn gap(&self, points: f32) -> Size<LengthPercentage> {
        let value = LengthPercentage::length(self.px(points));
        Size {
            width: value,
            height: value,
        }
    }
}
