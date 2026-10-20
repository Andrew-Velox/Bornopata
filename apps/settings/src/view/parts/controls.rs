//! The kinds of rows every page uses: a switch, a choice between a few values, a title, a notice.

use crate::i18n::t;
use bamboo_slip::taffy::{FlexDirection, LengthPercentage, NodeId, Rect, Style};
use bamboo_slip::{RenderError, Segmented, SettingRow, Switch, Ui, Visual, paragraph};
use borno_platform::Setting;

use crate::message::Message;
use crate::view::Page;

/// A page's title and the one sentence under it.
pub fn heading(
    ui: &mut Ui<Message>,
    page: &Page,
    title: &str,
    subtitle: &str,
) -> Result<NodeId, RenderError> {
    let theme = &page.theme;
    let name = ui.scene().node(
        Style::default(),
        Visual::Text {
            text: title.to_owned(),
            style: theme.text(theme.font.size + 9.0, 700, theme.colors.text),
        },
        &[],
    )?;
    let sub = paragraph(ui, theme, subtitle, Some(theme.colors.secondary))?;
    ui.scene().node(
        Style {
            flex_direction: FlexDirection::Column,
            gap: page.gap(4.0),
            padding: Rect {
                left: LengthPercentage::length(page.px(4.0)),
                right: LengthPercentage::length(0.0),
                top: LengthPercentage::length(0.0),
                bottom: LengthPercentage::length(page.px(8.0)),
            },
            ..Style::default()
        },
        Visual::Group,
        &[name, sub],
    )
}

/// Why the config file could not be read or written.
pub fn banner(ui: &mut Ui<Message>, page: &Page, text: &str) -> Result<NodeId, RenderError> {
    let sentence = format!("{} {text}", t("The settings file has a problem:"));
    paragraph(ui, &page.theme, &sentence, Some(page.theme.colors.accent))
}

/// A row with a switch: flipping it writes the opposite value into `[section] key`.
pub fn toggle(
    ui: &mut Ui<Message>,
    page: &Page,
    title: &str,
    info: &str,
    (section, key): (&'static str, &'static str),
    on: bool,
) -> Result<NodeId, RenderError> {
    let switch = Switch::new(on).build(
        ui,
        &page.theme,
        (section, key),
        Message::flag(section, key, !on),
    )?;
    SettingRow::new(title)
        .info(info)
        .control(switch)
        .build(ui, &page.theme)
}

/// One of a few values: `all` in the order shown, `label` the name on screen, `value` what is written to the file.
pub fn choice<T: Copy + PartialEq + 'static>(
    ui: &mut Ui<Message>,
    page: &Page,
    local: &str,
    all: &[T],
    current: T,
    label: fn(T) -> &'static str,
    message: fn(T) -> Message,
) -> Result<NodeId, RenderError> {
    let labels: Vec<&str> = all.iter().map(|value| label(*value)).collect();
    let selected = all.iter().position(|value| *value == current).unwrap_or(0);
    let values = all.to_vec();
    Segmented::new(&labels, selected)
        .build(ui, &page.theme, local, move |index| message(values[index]))
}

/// The value as a `Setting` for a text key.
pub fn text(section: &'static str, key: &'static str, value: &str) -> Message {
    Message::Set(section, key, Setting::Text(value.to_owned()))
}
