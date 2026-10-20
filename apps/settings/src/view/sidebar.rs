//! The sidebar: the logo and the name, a link for each page, and a small joint beside the current page.

use std::sync::Arc;

use bamboo_slip::taffy::{
    AlignItems, Dimension, FlexDirection, JustifyContent, LengthPercentage, LengthPercentageAuto,
    NodeId, Position, Rect, Size, Style,
};
use bamboo_slip::{
    BoxPaint, Fill, FontSpec, FontWeight, NavItem, RenderError, TITLEBAR, TextStyle, Ui, Visual,
};

use super::Page;
use crate::message::Message;
use crate::page_id::PageId;

/// The width of the sidebar (points).
pub const WIDTH: f32 = 208.0;

pub fn sidebar(ui: &mut Ui<Message>, page: &Page) -> Result<NodeId, RenderError> {
    let brand = brand(ui, page)?;
    let items = navigation(ui, page)?;
    let list = ui.scene().node(
        Style {
            flex_direction: FlexDirection::Column,
            gap: page.gap(1.0),
            margin: Rect {
                top: LengthPercentageAuto::length(page.px(16.0)),
                ..Rect::zero()
            },
            ..Style::default()
        },
        Visual::Group,
        &items,
    )?;
    let (vertical, horizontal) = (page.px(16.0), page.px(14.0));
    ui.scene().node(
        Style {
            size: Size {
                width: Dimension::length(page.px(WIDTH)),
                height: Dimension::auto(),
            },
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            gap: page.gap(14.0),
            padding: Rect {
                left: LengthPercentage::length(horizontal),
                right: LengthPercentage::length(horizontal),
                top: LengthPercentage::length(page.px(TITLEBAR + 14.0)),
                bottom: LengthPercentage::length(vertical),
            },
            ..Style::default()
        },
        Visual::solid(page.palette.sidebar, 0.0),
        &[brand, list],
    )
}

fn navigation(ui: &mut Ui<Message>, page: &Page) -> Result<Vec<NodeId>, RenderError> {
    let theme = &page.theme;
    ui.scope("nav", |ui| -> Result<Vec<NodeId>, RenderError> {
        let mut items = Vec::new();
        for id in PageId::ALL {
            let selected = id == page.settings.page;
            let mut item = NavItem::new(id.label())
                .icon(Arc::new(id.glyph()))
                .selected(selected);
            if selected {
                item = item.accessory(joint(ui, page)?);
            }
            items.push(item.build(ui, theme, id, Message::Page(id))?);
        }
        Ok(items)
    })
}

/// The blossom and the name.
fn brand(ui: &mut Ui<Message>, page: &Page) -> Result<NodeId, RenderError> {
    let mut children = Vec::new();
    if let Some(logo) = &page.settings.logo {
        let height = page.px(24.0);
        let (width, natural) = logo.size();
        children.push(ui.scene().node(
            Style {
                size: Size {
                    width: Dimension::length(height * width / natural),
                    height: Dimension::length(height),
                },
                ..Style::default()
            },
            Visual::Box(BoxPaint {
                fill: Some(Fill::Svg {
                    image: Arc::clone(logo),
                    slice: None,
                    px_per_unit: height / natural,
                }),
                border: None,
                radius: 0.0,
            }),
            &[],
        )?);
    }
    let font = FontSpec {
        weight: FontWeight(700),
        ..FontSpec::new(18.0, 24.0)
    };
    children.push(ui.scene().node(
        Style::default(),
        Visual::Text {
            text: "Bornopata".to_owned(),
            style: TextStyle::new(font.scaled(page.scale), 18.0, page.theme.colors.text, 1.0),
        },
        &[],
    )?);
    ui.scene().node(
        Style {
            flex_direction: FlexDirection::Row,
            align_items: Some(AlignItems::CENTER),
            gap: page.gap(8.0),
            padding: Rect {
                left: LengthPercentage::length(page.px(6.0)),
                ..Rect::zero()
            },
            ..Style::default()
        },
        Visual::Group,
        &children,
    )
}

/// The mark beside the current page: a short vertical bar, broken in the middle.
fn joint(ui: &mut Ui<Message>, page: &Page) -> Result<NodeId, RenderError> {
    let (thick, length) = (page.px(2.0), page.px(16.0));
    let part = length * 0.42;
    let mut pieces = Vec::new();
    for top in [0.0, length - part] {
        pieces.push(ui.scene().node(
            Style {
                position: Position::Absolute,
                size: Size {
                    width: Dimension::length(thick),
                    height: Dimension::length(part),
                },
                inset: Rect {
                    left: LengthPercentageAuto::length(0.0),
                    top: LengthPercentageAuto::length(top),
                    right: LengthPercentageAuto::auto(),
                    bottom: LengthPercentageAuto::auto(),
                },
                ..Style::default()
            },
            Visual::solid(page.theme.colors.accent, thick / 2.0),
            &[],
        )?);
    }
    let column = ui.scene().node(
        Style {
            size: Size {
                width: Dimension::length(thick),
                height: Dimension::length(length),
            },
            ..Style::default()
        },
        Visual::Group,
        &pieces,
    )?;
    ui.scene().node(
        Style {
            position: Position::Absolute,
            flex_direction: FlexDirection::Column,
            justify_content: Some(JustifyContent::CENTER),
            inset: Rect {
                left: LengthPercentageAuto::length(-page.px(9.0)),
                top: LengthPercentageAuto::length(0.0),
                right: LengthPercentageAuto::auto(),
                bottom: LengthPercentageAuto::length(0.0),
            },
            ..Style::default()
        },
        Visual::Group,
        &[column],
    )
}
