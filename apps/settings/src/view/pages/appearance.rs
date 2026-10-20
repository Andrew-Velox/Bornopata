//! The Appearance page: a preview of the candidate window and the choices that shape it.

use std::sync::Arc;

use bamboo_slip::taffy::{
    AlignItems, Dimension, FlexDirection, FlexWrap, JustifyContent, NodeId, Size, Style,
};
use bamboo_slip::{
    BoxPaint, Easing, Fill, Interact, RenderError, SettingRow, Switch, Ui, Visual, group,
    section_title,
};
use borno_platform::{PanelLayout, PanelMode};
use borno_render::{Pixmap, Theme};

use crate::message::Message;
use crate::settings::Settings;
use crate::view::Page;
use crate::view::parts::controls::{choice, heading, text};

/// A tile is at least this wide (points); as many fit in a row as there is room for, then they all grow to fill it.
const TILE: (f32, f32) = (150.0, 150.0);

const TILE_GAP: f32 = 12.0;

/// The side padding of the group the tiles sit in, see `bamboo_slip::group`.
const GROUP_PADDING: f32 = 16.0;

const TILE_PADDING: f32 = 10.0;

const PREVIEW_HEIGHT: f32 = 300.0;

const PREVIEW_PADDING: f32 = 14.0;

/// The theme the config names: a built-in one, or the theme folder it points at; Bornopata's own when neither works.
fn current_theme(settings: &Settings) -> Theme {
    let panel = &settings.config.panel;
    Theme::builtin(&panel.theme, panel.dark)
        .or_else(|| Theme::from_dir(std::path::Path::new(&panel.theme), panel.dark).ok())
        .unwrap_or_else(|| {
            Theme::builtin("bornopata", panel.dark).expect("the default theme exists")
        })
}

fn panel_mode_label(mode: PanelMode) -> &'static str {
    match mode {
        PanelMode::Auto => "Automatic",
        PanelMode::Popup => "Popup",
        PanelMode::Native => "Overlay",
        PanelMode::Fcitx => "Fcitx5 panel",
    }
}

fn panel_mode_key(mode: PanelMode) -> &'static str {
    match mode {
        PanelMode::Auto => "auto",
        PanelMode::Popup => "popup",
        PanelMode::Native => "native",
        PanelMode::Fcitx => "fcitx",
    }
}

fn layout_label(layout: PanelLayout) -> &'static str {
    match layout {
        PanelLayout::Vertical => "Vertical",
        PanelLayout::Horizontal => "Horizontal",
    }
}

pub fn appearance(ui: &mut Ui<Message>, page: &Page) -> Result<Vec<NodeId>, RenderError> {
    let theme = &page.theme;
    let panel = &page.settings.config.panel;
    let mut out = vec![
        heading(
            ui,
            page,
            "Appearance",
            "How the candidate window looks, and where it appears.",
        )?,
        preview(ui, page)?,
    ];

    let tiles = tiles(ui, page)?;
    let themes = SettingRow::new("Theme")
        .control(tiles)
        .stacked()
        .build(ui, theme)?;
    out.push(section_title(ui, theme, "Theme")?);
    out.push(group(ui, theme, &[themes])?);

    let layout = choice(
        ui,
        page,
        "layout",
        &[PanelLayout::Vertical, PanelLayout::Horizontal],
        panel.layout,
        layout_label,
        |value| {
            text(
                "panel",
                "layout",
                match value {
                    PanelLayout::Vertical => "vertical",
                    PanelLayout::Horizontal => "horizontal",
                },
            )
        },
    )?;
    let layout = SettingRow::new("Layout")
        .info("Vertical lists the candidates in a column, horizontal in a row.")
        .control(layout)
        .build(ui, theme)?;
    // Some themes are always light or always dark; for those the switch does nothing, so it is greyed out.
    let locked = current_theme(page.settings).locked_dark().is_some();
    let dark = Switch::new(panel.dark).disabled(locked).build(
        ui,
        theme,
        ("panel", "dark"),
        Message::flag("panel", "dark", !panel.dark),
    )?;
    let dark = SettingRow::new("Dark variant")
        .info("Themes that have a dark variant use it. Some themes are always light or always dark, and then this does nothing.")
        .control(dark)
        .build(ui, theme)?;
    out.push(section_title(ui, theme, "Layout")?);
    out.push(group(ui, theme, &[layout, dark])?);

    let mode = choice(
        ui,
        page,
        "mode",
        &[
            PanelMode::Auto,
            PanelMode::Popup,
            PanelMode::Native,
            PanelMode::Fcitx,
        ],
        panel.mode,
        panel_mode_label,
        |value| text("panel", "mode", panel_mode_key(value)),
    )?;
    let mode = SettingRow::new("Window")
        .info(
            "Automatic uses Bornopata's own window where the desktop allows it (a popup at the cursor in Wayland applications, a small overlay window in X11 ones) and Fcitx5's panel otherwise.",
        )
        .control(mode)
        .build(ui, theme)?;
    out.push(section_title(ui, theme, "Where it appears")?);
    out.push(group(ui, theme, &[mode])?);
    Ok(out)
}

/// The window as it is drawn now, in the chosen theme and layout, on a soft background.
fn preview(ui: &mut Ui<Message>, page: &Page) -> Result<NodeId, RenderError> {
    let settings = page.settings;
    let panel = &settings.config.panel;
    let theme = current_theme(settings);
    let fit = (
        page.content_width - page.px(PREVIEW_PADDING * 2.0),
        page.px(PREVIEW_HEIGHT - PREVIEW_PADDING * 2.0),
    );
    let image = settings.previews.borrow_mut().fitted(
        &theme,
        panel.layout == PanelLayout::Horizontal,
        7,
        page.scale,
        fit,
    );
    let mut children = Vec::new();
    if let Some(pixmap) = image {
        children.push(picture(ui, pixmap)?);
    }
    let colors = &page.theme.colors;
    ui.scene().node(
        Style {
            flex_direction: FlexDirection::Row,
            justify_content: Some(JustifyContent::CENTER),
            align_items: Some(AlignItems::CENTER),
            size: Size {
                width: Dimension::auto(),
                height: Dimension::length(page.px(PREVIEW_HEIGHT)),
            },
            flex_shrink: 0.0,
            ..Style::default()
        },
        Visual::Box(BoxPaint {
            fill: Some(Fill::Linear {
                angle: 120.0,
                stops: vec![(page.palette.hero[0], 0.0), (page.palette.hero[1], 0.8)],
            }),
            border: Some((page.px(1.0), colors.line)),
            radius: page.px(14.0),
        }),
        &children,
    )
}

/// A card for each built-in theme: a small picture and its name; the chosen one has an accent border.
fn tiles(ui: &mut Ui<Message>, page: &Page) -> Result<NodeId, RenderError> {
    let settings = page.settings;
    let panel = &settings.config.panel;
    let colors = page.theme.colors;
    let (width, height) = tile_size(page);
    let zoom = width / page.px(TILE.0);
    let mut cards = Vec::new();
    for listed in Theme::builtins() {
        let id = listed.id().to_owned();
        let shown = Theme::builtin(&id, panel.dark).unwrap_or_else(|| listed.clone());
        let key = ui.scope("themes", |ui| ui.key(&id));
        let state = ui.state(key);
        let selected = id == panel.theme;
        let fit = (
            width - page.px(TILE_PADDING * 2.0),
            height - page.px(TILE_PADDING * 2.0),
        );
        let image =
            settings
                .previews
                .borrow_mut()
                .fitted(&shown, false, 4, page.scale * 0.5 * zoom, fit);
        let mut tile_children = Vec::new();
        if let Some(pixmap) = image {
            tile_children.push(picture(ui, pixmap)?);
        }
        let border = if selected {
            (page.px(2.0), colors.accent)
        } else if state.hovered {
            (page.px(1.0), colors.secondary)
        } else {
            (page.px(1.0), colors.line)
        };
        let tile = ui.scene().node(
            Style {
                flex_direction: FlexDirection::Row,
                justify_content: Some(JustifyContent::CENTER),
                align_items: Some(AlignItems::CENTER),
                size: Size {
                    width: Dimension::length(width),
                    height: Dimension::length(height),
                },
                ..Style::default()
            },
            Visual::Box(BoxPaint {
                fill: Some(Fill::Solid(page.palette.tile)),
                border: Some(border),
                radius: page.px(12.0),
            }),
            &tile_children,
        )?;
        let (weight, color) = if selected {
            (600, colors.selected_text)
        } else {
            (400, colors.text)
        };
        let name = ui.scene().node(
            Style::default(),
            Visual::Text {
                text: listed.name().to_owned(),
                style: page.theme.text(page.theme.small, weight, color),
            },
            &[],
        )?;
        let card = ui.scene().node(
            Style {
                flex_direction: FlexDirection::Column,
                align_items: Some(AlignItems::CENTER),
                gap: page.gap(6.0),
                ..Style::default()
            },
            Visual::Group,
            &[tile, name],
        )?;
        ui.transition(tile, key, page.theme.hover, Easing::Ease);
        ui.interact(
            card,
            key,
            Interact::new()
                .on_click(text("panel", "theme", &id))
                .focusable(),
        );
        cards.push(card);
    }
    ui.scene().node(
        Style {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            gap: page.gap(TILE_GAP),
            ..Style::default()
        },
        Visual::Group,
        &cards,
    )
}

/// The size of a tile (pixels): as many as fit in a row, spread over the row.
fn tile_size(page: &Page) -> (f32, f32) {
    let available = page.content_width - page.px(GROUP_PADDING * 2.0);
    let (min, gap) = (page.px(TILE.0), page.px(TILE_GAP));
    let columns = ((available + gap) / (min + gap)).floor().max(1.0);
    let width = ((available - gap * (columns - 1.0)) / columns).max(min);
    (width, width * TILE.1 / TILE.0)
}

/// A bitmap at its own pixel size (it was drawn at the size it is shown).
fn picture(ui: &mut Ui<Message>, pixmap: Arc<Pixmap>) -> Result<NodeId, RenderError> {
    ui.scene().node(
        Style {
            size: Size {
                width: Dimension::length(pixmap.width() as f32),
                height: Dimension::length(pixmap.height() as f32),
            },
            flex_shrink: 0.0,
            ..Style::default()
        },
        Visual::Box(BoxPaint {
            fill: Some(Fill::Image {
                pixmap,
                slice: None,
                pixels_per_px: 1.0,
            }),
            border: None,
            radius: 0.0,
        }),
        &[],
    )
}
