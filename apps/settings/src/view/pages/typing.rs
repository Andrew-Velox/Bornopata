//! The Typing page: what is offered while you type.

use crate::i18n::t;
use bamboo_slip::taffy::NodeId;
use bamboo_slip::{RenderError, SettingRow, Ui, group, section_title};
use borno_platform::Language;

use crate::message::Message;
use crate::view::Page;
use crate::view::parts::controls::{choice, heading, text, toggle};

pub fn typing(ui: &mut Ui<Message>, page: &Page) -> Result<Vec<NodeId>, RenderError> {
    let theme = &page.theme;
    let general = &page.settings.config.general;
    let mut out = vec![heading(
        ui,
        page,
        t("Typing"),
        t("Changes are saved as you make them and take effect within a second."),
    )?];

    let language = choice(
        ui,
        page,
        "language",
        &[Language::Auto, Language::English, Language::Bangla],
        general.language,
        language_label,
        |value| text("general", "language", language_key(value)),
    )?;
    let language = SettingRow::new("Language · ভাষা")
        .info(t(
            "The language of this program. Automatic follows your system.",
        ))
        .control(language)
        .build(ui, theme)?;
    out.push(group(ui, theme, &[language])?);

    let rows = vec![
        toggle(
            ui,
            page,
            t("Remember my choices"),
            t(
                "A word you choose comes first the next time you type those letters. What it learns stays on this computer.",
            ),
            ("general", "learning"),
            general.learning,
        )?,
        toggle(
            ui,
            page,
            t("Bangla digits"),
            t("Type 123 and get ১২৩. Turn off to keep 1, 2, 3."),
            ("general", "bangla_digits"),
            general.bangla_digits,
        )?,
    ];
    out.push(section_title(ui, theme, t("Words"))?);
    out.push(group(ui, theme, &rows)?);

    let rows = vec![
        toggle(
            ui,
            page,
            t("English meaning"),
            t(
                "A short gloss such as “n. language” beside a word. It never changes the order of the candidates.",
            ),
            ("general", "gloss"),
            general.gloss,
        )?,
        toggle(
            ui,
            page,
            t("Emoji"),
            t("Emoji next to the words that name them: hridoy offers a heart, and heart does too."),
            ("general", "emoji"),
            general.emoji,
        )?,
        toggle(
            ui,
            page,
            t("English words"),
            t(
                "facebook types ফেসবুক and facebooke ফেসবুকে, instead of letters the Avro rules cannot spell.",
            ),
            ("general", "autocorrect"),
            general.autocorrect,
        )?,
        toggle(
            ui,
            page,
            t("Date and time words"),
            t("tarikh offers today's date, somoy the time and bar the weekday, in Bangla."),
            ("general", "shortcuts"),
            general.shortcuts,
        )?,
    ];
    out.push(section_title(ui, theme, t("Beside the candidates"))?);
    out.push(group(ui, theme, &rows)?);
    Ok(out)
}

fn language_label(language: Language) -> &'static str {
    match language {
        Language::Auto => t("Automatic"),
        Language::English => "English",
        Language::Bangla => "বাংলা",
    }
}

fn language_key(language: Language) -> &'static str {
    match language {
        Language::Auto => "auto",
        Language::English => "en",
        Language::Bangla => "bn",
    }
}
