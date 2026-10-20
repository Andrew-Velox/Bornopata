//! The Keys page: the two shortcuts Bornopata adds, recorded by pressing them, and the keys that cannot change.

use crate::i18n::t;
use bamboo_slip::taffy::{LengthPercentage, NodeId, Rect, Style};
use bamboo_slip::{
    Button, KeyRecorder, RenderError, SettingRow, Ui, Visual, group, paragraph, section_title,
};

use crate::message::Message;
use crate::shortcut::Shortcut;
use crate::view::Page;
use crate::view::parts::actions::with_notice;
use crate::view::parts::controls::heading;

/// The keys of a word being typed, which are not settings.
const FIXED: &str = "While you type a word: Space types the first candidate and a space, Enter types it and sends Enter, 1–9 choose a candidate, ↑ ↓ and Tab move the highlight, Backspace deletes a letter, Esc throws the word away. After a space: Tab takes a next-word suggestion, ↑ ↓ choose, Esc hides them.";

pub fn keys(ui: &mut Ui<Message>, page: &Page) -> Result<Vec<NodeId>, RenderError> {
    let theme = &page.theme;
    let mut out = vec![heading(
        ui,
        page,
        t("Keys"),
        t("Click a shortcut, then press the keys you want. Esc cancels."),
    )?];

    let mut rows = Vec::new();
    for shortcut in Shortcut::ALL {
        rows.push(recorder(ui, page, shortcut)?);
    }
    out.push(section_title(ui, theme, t("Shortcuts"))?);
    out.push(group(ui, theme, &rows)?);

    let reset = Button::new(t("Restore defaults")).build(
        ui,
        theme,
        "reset-shortcuts",
        Message::ResetShortcuts,
    )?;
    let reset = SettingRow::new(t("Both shortcuts"))
        .info(t("Back to Ctrl + Alt + Space and Shift + Delete."))
        .control(reset)
        .build(ui, theme)?;
    out.push(group(ui, theme, &[reset])?);

    let fixed = paragraph(ui, theme, t(FIXED), None)?;
    let inset = LengthPercentage::length(page.px(4.0));
    out.push(section_title(ui, theme, t("Keys that stay as they are"))?);
    out.push(ui.scene().node(
        Style {
            padding: Rect {
                left: inset,
                right: inset,
                top: LengthPercentage::length(0.0),
                bottom: LengthPercentage::length(0.0),
            },
            ..Style::default()
        },
        Visual::Group,
        &[fixed],
    )?);
    Ok(out)
}

/// A row for one shortcut: the title, an info mark, the recorder, and under it the reason a key was refused.
fn recorder(ui: &mut Ui<Message>, page: &Page, shortcut: Shortcut) -> Result<NodeId, RenderError> {
    let current = shortcut.current(&page.settings.config);
    let control = KeyRecorder::new(&current)
        .prompt(shortcut.prompt())
        .recording(page.settings.recording == Some(shortcut))
        .build(
            ui,
            &page.theme,
            ("recorder", shortcut),
            Message::Record(shortcut),
            move |press| Message::Recorded(shortcut, press.clone()),
            Message::StopRecording,
        )?;
    with_notice(
        ui,
        page,
        SettingRow::new(shortcut.title())
            .info(shortcut.info())
            .control(control),
        "shortcuts",
    )
}
