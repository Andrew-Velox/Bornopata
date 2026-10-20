//! The view: a sidebar and, beside it, the settings of the current page, the whole page scrolling.

pub mod icons;
mod page;
mod pages;
pub mod palette;
mod parts;
pub mod previews;
mod sidebar;

pub use page::Page;

use bamboo_slip::FontSpec;
use bamboo_slip::taffy::{Dimension, FlexDirection, LengthPercentage, NodeId, Rect, Size, Style};
use bamboo_slip::{
    Interact, KeyCode, RenderError, ScrollArea, TITLEBAR, Ui, Visual, WidgetColors, WidgetTheme,
};

use crate::message::Message;
use crate::page_id::PageId;
use crate::settings::Settings;
use palette::LIGHT;

/// The side margins of the content area (points).
const CONTENT_PADDING: f32 = 36.0;

/// A size up from the engine's default: 13 points is hard to read in a settings window.
fn widget_theme(colors: WidgetColors, scale: f32) -> WidgetTheme {
    WidgetTheme {
        font: FontSpec::new(15.0, 21.0),
        small: 12.5,
        control_height: 28.0,
        ..WidgetTheme::new(colors, scale)
    }
}

pub fn view(
    ui: &mut Ui<Message>,
    settings: &Settings,
    scale: f32,
    (width, height): (u32, u32),
) -> Result<NodeId, RenderError> {
    let palette = &LIGHT;
    let page = Page {
        settings,
        theme: widget_theme(palette.widgets, scale),
        palette,
        scale,
        content_width: width as f32 - (sidebar::WIDTH + CONTENT_PADDING * 2.0) * scale,
    };
    let sidebar = sidebar::sidebar(ui, &page)?;
    let content = content(ui, &page)?;
    let root = ui.scene().node(
        Style {
            size: Size {
                width: Dimension::length(width as f32),
                height: Dimension::length(height as f32),
            },
            flex_direction: FlexDirection::Row,
            ..Style::default()
        },
        Visual::solid(palette.window, 0.0),
        &[sidebar, content],
    )?;
    let key = ui.key("root");
    let recording = settings.recording.is_some();
    let dialog_open = settings.editor.is_some();
    ui.interact(
        root,
        key,
        Interact::new().on_key(move |press| {
            if press.key == KeyCode::Escape {
                if recording {
                    return Some(Message::StopRecording);
                }
                if dialog_open {
                    return Some(Message::ClosePhrase);
                }
            }
            let close = press.modifiers.command
                && matches!(&press.key, KeyCode::Character(text) if text == "w" || text == "q");
            close.then_some(Message::Quit)
        }),
    );
    Ok(root)
}

fn page_children(
    ui: &mut Ui<Message>,
    page: &Page,
    id: PageId,
) -> Result<Vec<NodeId>, RenderError> {
    Ok(match id {
        PageId::Typing => pages::typing::typing(ui, page)?,
        PageId::Keys => pages::keys::keys(ui, page)?,
        PageId::Appearance => pages::appearance::appearance(ui, page)?,
        PageId::Phrases => pages::phrases::phrases(ui, page)?,
        PageId::Learning => pages::learning::learning(ui, page)?,
        PageId::About => pages::about::about(ui, page)?,
    })
}

fn content(ui: &mut Ui<Message>, page: &Page) -> Result<NodeId, RenderError> {
    let mut children = Vec::new();
    if let Some(problem) = &page.settings.problem {
        children.push(parts::controls::banner(ui, page, problem)?);
    }
    children.extend(page_children(ui, page, page.settings.page)?);
    let column = ui.scene().node(
        Style {
            flex_direction: FlexDirection::Column,
            gap: page.gap(10.0),
            padding: Rect {
                left: LengthPercentage::length(page.px(CONTENT_PADDING)),
                right: LengthPercentage::length(page.px(CONTENT_PADDING)),
                top: LengthPercentage::length(page.px(TITLEBAR + 20.0)),
                bottom: LengthPercentage::length(page.px(28.0)),
            },
            ..Style::default()
        },
        Visual::Group,
        &children,
    )?;
    // Every page remembers its own scroll position.
    ScrollArea::new(Style {
        flex_grow: 1.0,
        ..Style::default()
    })
    .build(ui, &page.theme, ("content", page.settings.page), column)
}
