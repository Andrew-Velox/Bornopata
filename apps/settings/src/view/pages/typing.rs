//! The Typing page: what is offered while you type.

use bamboo_slip::taffy::NodeId;
use bamboo_slip::{RenderError, Ui, group, section_title};

use crate::message::Message;
use crate::view::Page;
use crate::view::parts::controls::{heading, toggle};

pub fn typing(ui: &mut Ui<Message>, page: &Page) -> Result<Vec<NodeId>, RenderError> {
    let theme = &page.theme;
    let general = &page.settings.config.general;
    let mut out = vec![heading(
        ui,
        page,
        "Typing",
        "Changes are saved as you make them and take effect within a second.",
    )?];

    let rows = vec![
        toggle(
            ui,
            page,
            "Remember my choices",
            "A word you choose comes first the next time you type those letters. What it learns stays on this computer.",
            ("general", "learning"),
            general.learning,
        )?,
        toggle(
            ui,
            page,
            "Bangla digits",
            "Type 123 and get ১২৩. Turn off to keep 1, 2, 3.",
            ("general", "bangla_digits"),
            general.bangla_digits,
        )?,
    ];
    out.push(section_title(ui, theme, "Words")?);
    out.push(group(ui, theme, &rows)?);

    let rows = vec![
        toggle(
            ui,
            page,
            "English meaning",
            "A short gloss such as “n. language” beside a word. It never changes the order of the candidates.",
            ("general", "gloss"),
            general.gloss,
        )?,
        toggle(
            ui,
            page,
            "Emoji",
            "Emoji next to the words that name them: hridoy offers a heart, and heart does too.",
            ("general", "emoji"),
            general.emoji,
        )?,
        toggle(
            ui,
            page,
            "Date and time words",
            "tarikh offers today's date, somoy the time and bar the weekday, in Bangla.",
            ("general", "shortcuts"),
            general.shortcuts,
        )?,
    ];
    out.push(section_title(ui, theme, "Beside the candidates")?);
    out.push(group(ui, theme, &rows)?);
    Ok(out)
}
