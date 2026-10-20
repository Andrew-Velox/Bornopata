//! The About page: the version, where things are, the licences, and the links.

use bamboo_slip::taffy::NodeId;
use bamboo_slip::{RenderError, SettingRow, Ui, group, paragraph, section_title};
use borno_platform::paths;

use crate::message::Message;
use crate::system::action::Action;
use crate::view::Page;
use crate::view::parts::actions::{actions, with_notice};
use crate::view::parts::controls::heading;

const LICENSE: &str = "Bornopata is free software under the GPL-3.0-or-later: you may use, change and share it, and what you share stays open. It grew out of the Qingjian input method.";

const DATA: &str = "The dictionary is built from Bengali Wikipedia and the Mozilla Common Voice sentences (CC BY-SA 4.0 and CC0). The English meanings come from Wiktionary through Kaikki.org (CC BY-SA 4.0). Emoji names are the Unicode CLDR annotations (Unicode License v3). The fallback word list is from upodesh and libavrophonetic (MIT).";

const PRIVACY: &str = "Nothing leaves this computer. Typing, the dictionary, learning, the meanings and emoji work offline, what you type is never written to a log, and password fields are left alone.";

pub fn about(ui: &mut Ui<Message>, page: &Page) -> Result<Vec<NodeId>, RenderError> {
    let theme = &page.theme;
    let mut out = vec![heading(
        ui,
        page,
        "About",
        &format!("Bornopata Settings {}", env!("CARGO_PKG_VERSION")),
    )?];

    let links = actions(
        ui,
        page,
        &[
            ("Website", Action::OpenWebsite),
            ("Source code", Action::OpenRepository),
        ],
    )?;
    let links = with_notice(
        ui,
        page,
        SettingRow::new("Bornopata")
            .info("An open-source Bangla phonetic input method.")
            .control(links),
        "links",
    )?;
    out.push(group(ui, theme, &[links])?);

    let buttons = actions(
        ui,
        page,
        &[
            ("Settings file", Action::OpenConfig),
            ("Data folder", Action::OpenDataDir),
            ("Logs", Action::OpenLogDir),
        ],
    )?;
    let files = with_notice(
        ui,
        page,
        SettingRow::new("Your files")
            .detail(&paths::user_dir().to_string_lossy())
            .control(buttons),
        "files",
    )?;
    out.push(section_title(ui, theme, "Files")?);
    out.push(group(ui, theme, &[files])?);

    out.push(section_title(ui, theme, "Licence")?);
    out.push(paragraph(ui, theme, LICENSE, None)?);
    out.push(section_title(ui, theme, "Data")?);
    out.push(paragraph(ui, theme, DATA, None)?);
    out.push(section_title(ui, theme, "Privacy")?);
    out.push(paragraph(ui, theme, PRIVACY, None)?);
    Ok(out)
}
