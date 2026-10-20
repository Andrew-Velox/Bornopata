//! A row of buttons, each one action, and the sentence that hangs under a row after one ran.

use bamboo_slip::taffy::{FlexDirection, NodeId, Style};
use bamboo_slip::{Button, RenderError, SettingRow, Ui, Visual, paragraph};

use crate::message::Message;
use crate::system::action::Action;
use crate::view::Page;

pub fn actions(
    ui: &mut Ui<Message>,
    page: &Page,
    buttons: &[(&str, Action)],
) -> Result<NodeId, RenderError> {
    let mut nodes = Vec::new();
    for &(label, action) in buttons {
        nodes.push(Button::new(label).build(
            ui,
            &page.theme,
            ("action", label),
            Message::Run(action),
        )?);
    }
    ui.scene().node(
        Style {
            flex_direction: FlexDirection::Row,
            gap: page.gap(8.0),
            ..Style::default()
        },
        Visual::Group,
        &nodes,
    )
}

/// Builds `row`, with the sentence about the last result under it when it belongs to this row.
pub fn with_notice(
    ui: &mut Ui<Message>,
    page: &Page,
    row: SettingRow,
    slot: &str,
) -> Result<NodeId, RenderError> {
    let mut row = row;
    if let Some((owner, text)) = &page.settings.notice
        && *owner == slot
    {
        row = row.below(paragraph(ui, &page.theme, text, None)?);
    }
    row.build(ui, &page.theme)
}
