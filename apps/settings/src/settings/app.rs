//! The link to bamboo-slip: build the window, handle a message.

use bamboo_slip::taffy::NodeId;
use bamboo_slip::{App, RenderError, TextEdit, Ui};
use borno_platform::Setting;

use super::{Learned, PhraseEditor, Phrases, Settings};
use crate::message::{FieldId, Message};
use crate::page_id::PageId;
use crate::shortcut::Shortcut;
use crate::system::action::Action;
use crate::view;

impl App for Settings {
    type Message = Message;

    fn view(
        &self,
        ui: &mut Ui<Message>,
        scale: f32,
        size: (u32, u32),
    ) -> Result<NodeId, RenderError> {
        view::view(ui, self, scale, size)
    }

    fn update(&mut self, message: Message) -> bool {
        let armed = self.armed.take();
        if !matches!(message, Message::Run(_)) {
            self.notice = None;
        }
        match message {
            Message::Page(page) => {
                self.page = page;
                self.recording = None;
                self.editor = None;
                if page == PageId::Learning {
                    self.learned = Learned::load();
                }
                if page == PageId::Phrases {
                    self.phrases = Phrases::load();
                }
            }
            Message::Set(section, key, setting) => self.write(section, key, setting),
            Message::Record(shortcut) => self.recording = Some(shortcut),
            Message::StopRecording => self.recording = None,
            Message::Recorded(shortcut, press) => match shortcut.accept(&press, &self.config) {
                Ok(value) => {
                    let (section, key) = shortcut.config_key();
                    self.write(section, key, Setting::Text(value));
                    self.recording = None;
                }
                Err(reason) => self.notice = Some(("shortcuts", reason)),
            },
            Message::ResetShortcuts => {
                for shortcut in Shortcut::ALL {
                    let (section, key) = shortcut.config_key();
                    self.write(
                        section,
                        key,
                        Setting::Text(shortcut.default_value().to_owned()),
                    );
                }
                self.recording = None;
            }
            Message::Run(action) => {
                // Forgetting is two presses: the first only turns the button into a confirmation.
                if action == Action::ForgetEverything && armed != Some(action) {
                    self.armed = Some(action);
                    self.notice = None;
                } else {
                    self.notice = action.run(&self.path).map(|text| (action.slot(), text));
                    if action == Action::ForgetEverything {
                        self.learned = Learned::default();
                    }
                }
            }
            Message::Edit(id, edit) => {
                if let Some(copied) = self
                    .fields
                    .get_mut(&id)
                    .and_then(|field| field.apply(&edit))
                {
                    bamboo_slip::set_clipboard(&copied);
                }
            }
            Message::OpenPhrase(index) => self.open_phrase(index),
            Message::SavePhrase => self.save_phrase(),
            Message::DeletePhrase => {
                let confirmed = self
                    .editor
                    .as_ref()
                    .is_some_and(|editor| editor.confirm_delete);
                if confirmed {
                    self.delete_phrase();
                } else if let Some(editor) = self.editor.as_mut() {
                    editor.confirm_delete = true;
                }
            }
            Message::ClosePhrase => self.editor = None,
            Message::Quit => return false,
        }
        true
    }
}

impl Settings {
    fn open_phrase(&mut self, index: Option<usize>) {
        let (code, text) = index
            .and_then(|index| self.phrases.entries().into_iter().nth(index))
            .map(|entry| (entry.code, entry.text))
            .unwrap_or_default();
        self.fields.insert(FieldId::PhraseCode, TextEdit::new(code));
        self.fields
            .insert(FieldId::PhraseText, TextEdit::multiline(text));
        self.editor = Some(PhraseEditor {
            index,
            confirm_delete: false,
            problem: None,
        });
    }

    fn save_phrase(&mut self) {
        let Some(editor) = self.editor.as_ref() else {
            return;
        };
        let text_of = |fields: &std::collections::HashMap<FieldId, TextEdit>, id| {
            fields
                .get(&id)
                .map(|edit| edit.text().to_owned())
                .unwrap_or_default()
        };
        let code = text_of(&self.fields, FieldId::PhraseCode);
        let text = text_of(&self.fields, FieldId::PhraseText);
        match self
            .phrases
            .set(editor.index, &code, text.trim_end_matches('\n'))
        {
            Ok(()) => self.editor = None,
            Err(problem) => {
                if let Some(editor) = self.editor.as_mut() {
                    editor.problem = Some(problem);
                }
            }
        }
    }

    fn delete_phrase(&mut self) {
        if let Some(index) = self.editor.as_ref().and_then(|editor| editor.index)
            && let Err(problem) = self.phrases.remove(index)
        {
            self.notice = Some(("phrases", problem));
        }
        self.editor = None;
    }
}
