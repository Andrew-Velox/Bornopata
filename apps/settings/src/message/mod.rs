//! What a control sends: mostly "set this setting to that", the rest is the program's own state and its buttons.

mod field_id;

pub use field_id::FieldId;

use bamboo_slip::{Edit, KeyPress};
use borno_platform::Setting;

use crate::page_id::PageId;
use crate::shortcut::Shortcut;
use crate::system::action::Action;

#[derive(Debug, Clone)]
pub enum Message {
    Page(PageId),

    /// Changes `[section] key` in the config file.
    Set(&'static str, &'static str, Setting),

    /// Starts recording this shortcut.
    Record(Shortcut),

    /// A key pressed while recording.
    Recorded(Shortcut, KeyPress),

    /// Escape or a click elsewhere: stop recording.
    StopRecording,

    /// Puts both shortcuts back to their defaults.
    ResetShortcuts,

    Run(Action),

    /// Typing in a text field.
    Edit(FieldId, Edit),

    /// Opens the phrase card: `None` adds one, `Some(i)` edits the i-th.
    OpenPhrase(Option<usize>),

    SavePhrase,

    /// The card's Delete; the first press only asks to confirm.
    DeletePhrase,

    ClosePhrase,

    Quit,
}

impl Message {
    pub fn flag(section: &'static str, key: &'static str, on: bool) -> Self {
        Self::Set(section, key, Setting::Bool(on))
    }
}
