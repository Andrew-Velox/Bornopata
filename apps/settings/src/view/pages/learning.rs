//! The Learning page: how much it has learned, and a way to start over.

use crate::i18n::t;
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
        t("Learning"),
        t(
            "What Bornopata remembers about your typing. It all stays on this computer, in plain text files.",
        ),
    )?];

    let words = SettingRow::new(t("Words"))
        .detail(&count(learned.words))
        .build(ui, theme)?;
    let choices = SettingRow::new(t("Choices"))
        .detail(&format!(
            "{} {}",
            count(learned.choices),
            t("(which word you picked for which letters)")
        ))
        .build(ui, theme)?;
    let pairs = SettingRow::new(t("Word order"))
        .detail(&format!(
            "{} {}",
            count(learned.pairs),
            t("(which word came after which)")
        ))
        .build(ui, theme)?;
    out.push(section_title(ui, theme, t("So far"))?);
    out.push(group(ui, theme, &[words, choices, pairs])?);

    let remember = toggle(
        ui,
        page,
        t("Remember my choices"),
        t(
            "Turn off to stop learning. What was learned stays and still helps, until you forget it below.",
        ),
        ("general", "learning"),
        settings.config.general.learning,
    )?;
    let forget_label = if settings.armed == Some(Action::ForgetEverything) {
        t("Press again to forget everything")
    } else {
        t("Forget everything")
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
        SettingRow::new(t("Start again"))
            .info(t("Removes every learned word, choice and word order. Your phrases and settings are not touched."))
            .control(forget),
        "forget",
    )?;
    out.push(section_title(ui, theme, t("Control"))?);
    out.push(group(ui, theme, &[remember, forget])?);

    let folder = actions(ui, page, &[(t("Open the folder"), Action::OpenDataDir)])?;
    let files = with_notice(
        ui,
        page,
        SettingRow::new(t("The files"))
            .info(t("user-words.tsv, user-choices.tsv, user-ngram.tsv and user-hidden.tsv. You can read them in any text editor."))
            .control(folder),
        "files",
    )?;
    out.push(group(ui, theme, &[files])?);
    Ok(out)
}

fn count(n: usize) -> String {
    match n {
        0 => t("nothing yet").to_owned(),
        n => n.to_string(),
    }
}
