//! The Phrases page: your own codes that type longer texts.

use bamboo_slip::taffy::{
    AlignItems, Dimension, FlexDirection, JustifyContent, LengthPercentage, LengthPercentageAuto,
    NodeId, Rect, Size, Style,
};
use bamboo_slip::{
    Button, RenderError, SettingRow, TextField, Ui, Visual, dialog, group, paragraph, section_title,
};

use crate::message::{FieldId, Message};
use crate::settings::PhraseEntry;
use crate::system::action::Action;
use crate::view::Page;
use crate::view::parts::actions::{actions, with_notice};
use crate::view::parts::controls::heading;

/// How many characters of a phrase the list shows.
const PREVIEW: usize = 36;

const DIALOG_WIDTH: f32 = 460.0;

pub fn phrases(ui: &mut Ui<Message>, page: &Page) -> Result<Vec<NodeId>, RenderError> {
    let theme = &page.theme;
    let mut out = vec![heading(
        ui,
        page,
        "Phrases",
        "Type a short code and your own text comes first in the list. Phrases are never learned.",
    )?];

    let entries = page.settings.phrases.entries();
    let mut rows = Vec::new();
    if entries.is_empty() {
        rows.push(paragraph(
            ui,
            theme,
            "No phrases yet. Add one, for example mymail for your email address.",
            Some(theme.colors.secondary),
        )?);
    }
    for (index, entry) in entries.iter().enumerate() {
        rows.push(phrase_row(ui, page, index, entry)?);
    }
    let add = Button::new("Add a phrase…").primary().build(
        ui,
        theme,
        "add-phrase",
        Message::OpenPhrase(None),
    )?;
    rows.push(with_notice(
        ui,
        page,
        SettingRow::new("New phrase")
            .info("A code of Roman letters, matched exactly as you type it. In the text, a new line is kept as a line break.")
            .control(add),
        "phrases",
    )?);
    out.push(section_title(ui, theme, "Your phrases")?);
    out.push(group(ui, theme, &rows)?);

    let file = actions(ui, page, &[("Open the file", Action::OpenPhrasesFile)])?;
    let file = with_notice(
        ui,
        page,
        SettingRow::new("The phrases file")
            .info("phrases.tsv, one phrase per line: the code, a Tab, then the text. Edits made there show up here when you open this page again.")
            .control(file),
        "files",
    )?;
    out.push(group(ui, theme, &[file])?);

    if page.settings.editor.is_some() {
        editor(ui, page)?;
    }
    Ok(out)
}

fn phrase_row(
    ui: &mut Ui<Message>,
    page: &Page,
    index: usize,
    entry: &PhraseEntry,
) -> Result<NodeId, RenderError> {
    let theme = &page.theme;
    let code = ui.scene().node(
        Style {
            flex_shrink: 0.0,
            ..Style::default()
        },
        Visual::Text {
            text: entry.code.clone(),
            style: theme.text(theme.font.size, 600, theme.colors.text),
        },
        &[],
    )?;
    let text = ui.scene().node(
        Style::default(),
        Visual::Text {
            text: preview(&entry.text),
            style: theme.text(theme.small, 400, theme.colors.secondary),
        },
        &[],
    )?;
    let edit = Button::new("Edit").build(
        ui,
        theme,
        ("phrase-edit", index),
        Message::OpenPhrase(Some(index)),
    )?;
    let trailing = ui.scene().node(
        Style {
            flex_direction: FlexDirection::Row,
            align_items: Some(AlignItems::CENTER),
            gap: page.gap(12.0),
            ..Style::default()
        },
        Visual::Group,
        &[text, edit],
    )?;
    let space = LengthPercentage::length(page.px(10.0));
    ui.scene().node(
        Style {
            flex_direction: FlexDirection::Row,
            align_items: Some(AlignItems::CENTER),
            justify_content: Some(JustifyContent::SPACE_BETWEEN),
            gap: page.gap(16.0),
            min_size: Size {
                width: LengthPercentageAuto::auto(),
                height: LengthPercentageAuto::length(page.px(theme.control_height + 20.0)),
            },
            padding: Rect {
                top: space,
                bottom: space,
                left: LengthPercentage::length(0.0),
                right: LengthPercentage::length(0.0),
            },
            ..Style::default()
        },
        Visual::Group,
        &[code, trailing],
    )
}

/// The card for adding or editing a phrase.
fn editor(ui: &mut Ui<Message>, page: &Page) -> Result<(), RenderError> {
    let theme = &page.theme;
    let Some(editor) = &page.settings.editor else {
        return Ok(());
    };
    let fields = &page.settings.fields;
    let (Some(code), Some(text)) = (
        fields.get(&FieldId::PhraseCode),
        fields.get(&FieldId::PhraseText),
    ) else {
        return Ok(());
    };
    let title = ui.scene().node(
        Style::default(),
        Visual::Text {
            text: if editor.index.is_some() {
                "Edit phrase"
            } else {
                "New phrase"
            }
            .to_owned(),
            style: theme.text(theme.font.size + 2.0, 600, theme.colors.text),
        },
        &[],
    )?;
    let code = TextField::new(code)
        .placeholder("letters only, like mymail")
        .width(220.0)
        .build(
            ui,
            theme,
            "phrase-code",
            |edit| Message::Edit(FieldId::PhraseCode, edit),
            None,
            Some(Message::ClosePhrase),
        )?;
    let code = SettingRow::new("Code").control(code).build(ui, theme)?;
    let content = TextField::new(text)
        .placeholder("The text that is typed. It can run over several lines.")
        .width(DIALOG_WIDTH - 40.0)
        .rows(4)
        .build(
            ui,
            theme,
            "phrase-text",
            |edit| Message::Edit(FieldId::PhraseText, edit),
            None,
            Some(Message::ClosePhrase),
        )?;
    let mut children = vec![title, code, content];
    if let Some(problem) = &editor.problem {
        children.push(paragraph(ui, theme, problem, Some(theme.colors.accent))?);
    }
    let mut buttons = Vec::new();
    if editor.index.is_some() {
        buttons.push(
            Button::new(if editor.confirm_delete {
                "Press again to delete"
            } else {
                "Delete"
            })
            .build(ui, theme, "phrase-delete", Message::DeletePhrase)?,
        );
    }
    buttons.push(ui.scene().node(
        Style {
            flex_grow: 1.0,
            size: Size {
                width: Dimension::auto(),
                height: Dimension::auto(),
            },
            ..Style::default()
        },
        Visual::Group,
        &[],
    )?);
    buttons.push(Button::new("Cancel").build(ui, theme, "phrase-cancel", Message::ClosePhrase)?);
    buttons.push(Button::new("Save").primary().build(
        ui,
        theme,
        "phrase-save",
        Message::SavePhrase,
    )?);
    children.push(ui.scene().node(
        Style {
            flex_direction: FlexDirection::Row,
            align_items: Some(AlignItems::CENTER),
            gap: page.gap(8.0),
            margin: Rect {
                top: LengthPercentageAuto::length(page.px(4.0)),
                ..Rect::zero()
            },
            ..Style::default()
        },
        Visual::Group,
        &buttons,
    )?);
    dialog(ui, theme, "phrase-dialog", DIALOG_WIDTH, &children)
}

/// The first line of a phrase, cut short.
fn preview(text: &str) -> String {
    let first = text.lines().next().unwrap_or("");
    let mut shown: String = first.chars().take(PREVIEW).collect();
    if first.chars().count() > PREVIEW || text.lines().count() > 1 {
        shown.push('…');
    }
    shown
}
