//! Helpers: a Router over a fixed word list and shorthand for the messages tests send.

use std::sync::Arc;

use borno_learning::FrequencyLearner;
use borno_linux_server::{Router, Shared};
use borno_platform::Config;
use borno_platform::protocol::{
    Capabilities, ClientMessage, Event, Frame, Identity, KeyEvent, KeyModifiers, KeyOutcome,
    ServerMessage, SessionId,
};
use borno_transliteration::{FixedWordSource, UpodeshWordSource};

pub const SESSION: SessionId = SessionId(1);

pub const BACKSPACE: u32 = 0x08;
pub const TAB: u32 = 0x09;
pub const ENTER: u32 = 0x0d;
pub const ESCAPE: u32 = 0x1b;
pub const UP: u32 = 0x26;
pub const DOWN: u32 = 0x28;
pub const LEFT: u32 = 0x25;

/// What a reply to an event contained.
#[derive(Debug)]
pub struct Reply {
    pub outcome: KeyOutcome,
    pub commit: Option<String>,
    pub identity: Identity,
    pub frame: Frame,
}

pub struct Rig {
    pub router: Router,
}

impl Rig {
    /// The upodesh scaffold list, learning in memory.
    pub fn scaffold() -> Self {
        let shared = Shared::new(
            Arc::new(UpodeshWordSource::new()),
            Some(FrequencyLearner::in_memory()),
            None,
        );
        Self::with(Config::default(), shared)
    }

    pub fn fixed(entries: &[(&str, &[&str])]) -> Self {
        let entries = entries.iter().map(|(roman, words)| {
            (
                (*roman).to_owned(),
                words.iter().map(|word| (*word).to_owned()).collect(),
            )
        });
        let shared = Shared::new(
            Arc::new(FixedWordSource::new(entries)),
            Some(FrequencyLearner::in_memory()),
            None,
        );
        Self::with(Config::default(), shared)
    }

    pub fn with(config: Config, shared: Shared) -> Self {
        let mut rig = Self {
            router: Router::new(config, shared),
        };
        let opened = rig.router.handle(ClientMessage::Open {
            session: SESSION,
            app: "test".into(),
            protocol: borno_platform::protocol::PROTOCOL_VERSION,
            generation: 1,
            context: "ctx".into(),
        });
        assert!(matches!(opened, Some(ServerMessage::Opened { .. })));
        rig.event(Event::Capabilities(Capabilities::default()));
        rig
    }

    pub fn event(&mut self, event: Event) -> Reply {
        match self.router.handle(ClientMessage::Event {
            session: SESSION,
            event,
        }) {
            Some(ServerMessage::Result {
                outcome,
                commit,
                identity,
                frame,
                ..
            }) => Reply {
                outcome,
                commit,
                identity,
                frame,
            },
            other => panic!("expected a Result, got {other:?}"),
        }
    }

    pub fn key(&mut self, event: KeyEvent) -> Reply {
        self.event(Event::Key {
            event,
            release: false,
        })
    }

    pub fn release(&mut self, event: KeyEvent) -> Reply {
        self.event(Event::Key {
            event,
            release: true,
        })
    }

    /// A printable character key.
    pub fn type_char(&mut self, ch: char) -> Reply {
        self.key(char_key(ch))
    }

    /// Types every character, returning the text committed along the way.
    pub fn type_text(&mut self, text: &str) -> String {
        let mut committed = String::new();
        for ch in text.chars() {
            if let Some(text) = self.type_char(ch).commit {
                committed.push_str(&text);
            }
        }
        committed
    }

    pub fn function(&mut self, code: u32) -> Reply {
        self.key(function_key(code, KeyModifiers::default()))
    }
}

pub fn char_key(ch: char) -> KeyEvent {
    KeyEvent::new(ch as u32, Some(ch), KeyModifiers::default())
}

pub fn function_key(code: u32, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, None, modifiers)
}
