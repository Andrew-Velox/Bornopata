//! The program's state: the config file (the one source of truth), the current page and what is open on it. A change
//! is written straight into the config file and read back; the Server notices the saved file and starts over.

mod app;
mod config;
mod learned;
mod phrases;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use bamboo_slip::{SvgImage, TextEdit};
use borno_platform::Config;

use crate::message::FieldId;
use crate::page_id::PageId;
use crate::shortcut::Shortcut;
use crate::system::action::Action;
use crate::view::previews::Previews;

pub use learned::Learned;
pub use phrases::{Entry as PhraseEntry, Phrases};

/// The phrase card: which phrase it edits and whether Delete has been pressed once.
pub struct PhraseEditor {
    pub index: Option<usize>,
    pub confirm_delete: bool,
    pub problem: Option<String>,
}

pub struct Settings {
    path: PathBuf,

    pub config: Config,

    /// Why the config file could not be read or written, when it could not.
    pub problem: Option<String>,

    pub page: PageId,

    pub previews: RefCell<Previews>,

    pub logo: Option<Arc<SvgImage>>,

    /// The shortcut being recorded.
    pub recording: Option<Shortcut>,

    /// A sentence about what a button did, and the row it hangs under.
    pub notice: Option<(&'static str, String)>,

    /// A button pressed once and waiting for a second press to confirm.
    pub armed: Option<Action>,

    pub phrases: Phrases,

    pub editor: Option<PhraseEditor>,

    pub fields: HashMap<FieldId, TextEdit>,

    pub learned: Learned,
}

impl Settings {
    pub fn load(path: PathBuf) -> Result<Self, borno_platform::ConfigError> {
        let (config, problem) = match Config::load(&path) {
            Ok(config) => (config, None),
            Err(error) => (Config::default(), Some(error.to_string())),
        };
        let logo = SvgImage::parse(include_bytes!("../../assets/logo.svg"))
            .map(Arc::new)
            .inspect_err(|error| tracing::warn!(%error, "the logo does not parse"))
            .ok();
        Ok(Self {
            path,
            config,
            problem,
            page: PageId::Typing,
            previews: RefCell::new(Previews::new()),
            logo,
            recording: None,
            notice: None,
            armed: None,
            phrases: Phrases::load(),
            editor: None,
            fields: HashMap::new(),
            learned: Learned::load(),
        })
    }
}
