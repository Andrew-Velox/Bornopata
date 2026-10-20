//! The Learning page: how much it has learned, and a way to start over.

use bamboo_slip::taffy::NodeId;
use bamboo_slip::{Button, RenderError, SettingRow, Ui, group, section_title};

use crate::message::Message;
use crate::system::action::Action;
use crate::view::Page;
use crate::view::parts::actions::{actions, with_notice};
use crate::view::parts::controls::{heading, toggle};

pub fn learning(ui: &mut Ui<Message>, page: &Page) -> Result<Vec<NodeId>, RenderError> {
    let theme = &page.theme;
    let settings = page.settings;
    let learned = settings.learned;
    let mut out = vec![heading(
        ui,
        page,
        "Learning",
        "What Bornopata remembers about your typing. It all stays on this computer, in plain text files.",
    )?];

    let words = SettingRow::new("Words")
        .detail(&count(learned.words))
        .build(ui, theme)?;
    let choices = SettingRow::new("Choices")
        .detail(&format!(
            "{} (which word you picked for which letters)",
            count(learned.choices)
        ))
        .build(ui, theme)?;
    let pairs = SettingRow::new("Word order")
        .detail(&format!(
            "{} (which word came after which)",
            count(learned.pairs)
        ))
        .build(ui, theme)?;
    out.push(section_title(ui, theme, "So far")?);
    out.push(group(ui, theme, &[words, choices, pairs])?);

    let remember = toggle(
        ui,
        page,
        "Remember my choices",
        "Turn off to stop learning. What was learned stays and still helps, until you forget it below.",
        ("general", "learning"),
        settings.config.general.learning,
    )?;
    let forget_label = if settings.armed == Some(Action::ForgetEverything) {
        "Press again to forget everything"
    } else {
        "Forget everything"
    };
    let forget = Button::new(forget_label)
        .disabled(learned.is_empty() && settings.armed != Some(Action::ForgetEverything))
        .build(
            ui,
            theme,
            "forget-everything",
            Message::Run(Action::ForgetEverything),
        )?;
    let forget = with_notice(
        ui,
        page,
        SettingRow::new("Start again")
            .info("Removes every learned word, choice and word order. Your phrases and settings are not touched.")
            .control(forget),
        "forget",
    )?;
    out.push(section_title(ui, theme, "Control")?);
    out.push(group(ui, theme, &[remember, forget])?);

    let folder = actions(ui, page, &[("Open the folder", Action::OpenDataDir)])?;
    let files = with_notice(
        ui,
        page,
        SettingRow::new("The files")
            .info("user-words.tsv, user-choices.tsv and user-ngram.tsv. You can read them in any text editor.")
            .control(folder),
        "files",
    )?;
    out.push(group(ui, theme, &[files])?);
    Ok(out)
}

fn count(n: usize) -> String {
    match n {
        0 => "nothing yet".to_owned(),
        n => n.to_string(),
    }
}
