//! The colors of the program: white and ink with the plum of the website, a soft rose for selection.

use bamboo_slip::{Color, WidgetColors};

pub struct Palette {
    pub widgets: WidgetColors,

    pub window: Color,

    pub sidebar: Color,

    /// The preview card's gradient (top left to bottom right).
    pub hero: [Color; 2],

    /// The background of a theme tile.
    pub tile: Color,
}

pub const LIGHT: Palette = Palette {
    widgets: WidgetColors {
        text: Color::rgb(0x1b, 0x1b, 0x1f),
        secondary: Color::rgb(0x7b, 0x7b, 0x85),
        accent: Color::rgb(0x4d, 0x2f, 0x4f),
        on_accent: Color::rgb(0xff, 0xff, 0xff),
        well: Color::rgb(0xf1, 0xf0, 0xf2),
        well_hover: Color::rgb(0xe8, 0xe6, 0xea),
        chip: Color::rgb(0xff, 0xff, 0xff),
        chip_text: Color::rgb(0x4d, 0x2f, 0x4f),
        surface: Color::rgb(0xff, 0xff, 0xff),
        popup: Color::rgb(0xff, 0xff, 0xff),
        line: Color::rgb(0xe9, 0xe7, 0xec),
        hover: Color::rgba(0x4d, 0x2f, 0x4f, 14),
        pressed: Color::rgba(0x4d, 0x2f, 0x4f, 30),
        selected: Color::rgba(0xc4, 0x78, 0x9f, 46),
        selected_text: Color::rgb(0x4d, 0x2f, 0x4f),
        focus: Color::rgba(0xc4, 0x78, 0x9f, 220),
        knob: Color::rgb(0xff, 0xff, 0xff),
        button: Color::rgb(0xff, 0xff, 0xff),
        shadow: Color::gray(0, 50),
    },
    window: Color::rgb(0xfa, 0xfa, 0xfb),
    sidebar: Color::rgb(0xf3, 0xf2, 0xf5),
    hero: [Color::rgb(0xfb, 0xee, 0xf3), Color::rgb(0xff, 0xff, 0xff)],
    tile: Color::rgb(0xfa, 0xf8, 0xf9),
};
