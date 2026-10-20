//! The About page: the version, where things are, the licences, and the links.

use crate::i18n::t;
use bamboo_slip::taffy::NodeId;
use bamboo_slip::{RenderError, SettingRow, Ui, group, paragraph, section_title};
use borno_platform::paths;

use crate::message::Message;
use crate::system::action::Action;
use crate::view::Page;
use crate::view::parts::actions::{actions, with_notice};
use crate::view::parts::controls::heading;

const LICENSE: &str = "Bornopata is free software under the GPL-3.0-or-later: you may use, change and share it, and what you share stays open. It grew out of the Qingjian input method.";

const DATA: &str = "The dictionary is built from Bengali Wikipedia and the Mozilla Common Voice sentences (CC BY-SA 4.0 and CC0). The next-word model counts word pairs in the same sentences. The English meanings come from Wiktionary through Kaikki.org (CC BY-SA 4.0). Emoji names are the Unicode CLDR annotations (Unicode License v3). The fallback word list is from upodesh and libavrophonetic (MIT).";

const PRIVACY: &str = "Nothing leaves this computer. Typing, the dictionary, learning, the meanings and emoji work offline, what you type is never written to a log, and password fields are left alone.";

pub fn about(ui: &mut Ui<Message>, page: &Page) -> Result<Vec<NodeId>, RenderError> {
    let theme = &page.theme;
    let mut out = vec![heading(
        ui,
        page,
        t("About"),
        &format!("{} {}", t("Bornopata Settings"), env!("CARGO_PKG_VERSION")),
    )?];

    let links = actions(
        ui,
        page,
        &[
            (t("Website"), Action::OpenWebsite),
            (t("Source code"), Action::OpenRepository),
        ],
    )?;
    let links = with_notice(
        ui,
        page,
        SettingRow::new("Bornopata")
            .info(t("An open-source Bangla phonetic input method."))
            .control(links),
        "links",
    )?;
    out.push(group(ui, theme, &[links])?);

    let buttons = actions(
        ui,
        page,
        &[
            (t("Settings file"), Action::OpenConfig),
            (t("Data folder"), Action::OpenDataDir),
            (t("Logs"), Action::OpenLogDir),
        ],
    )?;
    let files = with_notice(
        ui,
        page,
        SettingRow::new(t("Your files"))
            .detail(&paths::user_dir().to_string_lossy())
            .control(buttons),
        "files",
    )?;
    out.push(section_title(ui, theme, t("Files"))?);
    out.push(group(ui, theme, &[files])?);

    out.push(section_title(ui, theme, t("Licence"))?);
    out.push(paragraph(ui, theme, t(LICENSE), None)?);
    out.push(section_title(ui, theme, t("Data"))?);
    out.push(paragraph(ui, theme, t(DATA), None)?);
    out.push(section_title(ui, theme, t("Privacy"))?);
    out.push(paragraph(ui, theme, t(PRIVACY), None)?);
    Ok(out)
}
