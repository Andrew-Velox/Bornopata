//! The pin shortcut and the pinned window: a pinned window is used even where the popup would be, and without any
//! cursor.

mod support;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use borno_linux_server::Shared;
use borno_linux_server::panel::PanelSink;
use borno_linux_server::popup::PopupSink;
use borno_platform::Config;
use borno_platform::protocol::{CursorRect, KeyEvent, KeyModifiers, KeyOutcome};
use borno_transliteration::FixedWordSource;
use support::*;

const TOKEN: u64 = 5;

#[derive(Clone, Default)]
struct FakeWindow {
    pinned: Arc<Mutex<bool>>,
    shown: Arc<Mutex<Vec<CursorRect>>>,
}

impl PanelSink for FakeWindow {
    fn show(&mut self, _: borno_render::Frame, cursor: CursorRect) -> bool {
        self.shown.lock().unwrap().push(cursor);
        true
    }

    fn hide(&mut self) {}

    fn pinned(&self) -> bool {
        *self.pinned.lock().unwrap()
    }

    fn toggle_pin(&mut self) {
        let mut pinned = self.pinned.lock().unwrap();
        *pinned = !*pinned;
    }
}

#[derive(Clone, Default)]
struct FakePopup {
    shown: Arc<Mutex<usize>>,
}

impl PopupSink for FakePopup {
    fn show(&mut self, _: u64, _: &str, _: borno_render::Frame) -> bool {
        *self.shown.lock().unwrap() += 1;
        true
    }

    fn hide(&mut self, _: u64) {}

    fn attached(&self, token: u64) -> bool {
        HashSet::from([TOKEN]).contains(&token)
    }
}

fn rig(window: &FakeWindow, popup: &FakePopup) -> Rig {
    let words = FixedWordSource::new([("kotha".to_owned(), vec!["কথা".to_owned()])]);
    let shared = Shared::new(Arc::new(words), None, None);
    Rig::with_surfaces(
        Config::default(),
        shared,
        Some(Box::new(window.clone())),
        Some(Box::new(popup.clone())),
        Some(TOKEN),
    )
}

fn pin_key() -> KeyEvent {
    let modifiers = KeyModifiers {
        ctrl: true,
        alt: true,
        ..KeyModifiers::default()
    };
    KeyEvent::new('m' as u32, Some('m'), modifiers)
}

#[test]
fn the_popup_is_used_until_the_window_is_pinned() {
    let (window, popup) = (FakeWindow::default(), FakePopup::default());
    let mut rig = rig(&window, &popup);
    rig.type_text("kot");
    assert_eq!(*popup.shown.lock().unwrap(), 3);
    assert!(window.shown.lock().unwrap().is_empty());
}

#[test]
fn the_shortcut_moves_the_word_from_the_popup_to_the_pinned_window_and_back() {
    let (window, popup) = (FakeWindow::default(), FakePopup::default());
    let mut rig = rig(&window, &popup);
    rig.type_text("kot");
    let reply = rig.key(pin_key());
    assert_eq!(reply.outcome, KeyOutcome::Consumed);
    assert!(reply.frame.native);
    // No cursor was ever reported, and the window shows anyway.
    assert_eq!(window.shown.lock().unwrap().len(), 1);
    assert_eq!(rig.type_char('h').frame.roman, "koth");
    assert_eq!(window.shown.lock().unwrap().len(), 2);
    rig.key(pin_key());
    let popup_before = *popup.shown.lock().unwrap();
    rig.type_char('a');
    assert_eq!(*popup.shown.lock().unwrap(), popup_before + 1);
}

#[test]
fn the_shortcut_does_nothing_when_there_is_no_window_to_pin() {
    let (window, popup) = (FakeWindow::default(), FakePopup::default());
    let mut rig = rig(&window, &popup);
    let reply = rig.key(pin_key());
    assert_eq!(reply.outcome, KeyOutcome::Passthrough);
    assert!(!*window.pinned.lock().unwrap());
}
