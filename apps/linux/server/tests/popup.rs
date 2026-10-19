//! The input-method popup: when the Router shows a frame in it instead of the window, and when the shell must draw.

mod support;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use borno_linux_server::Shared;
use borno_linux_server::panel::PanelSink;
use borno_linux_server::popup::PopupSink;
use borno_platform::Config;
use borno_platform::protocol::{ClientMessage, CursorRect, Event};
use borno_transliteration::FixedWordSource;
use support::*;

const TOKEN: u64 = 77;

#[derive(Debug, PartialEq)]
enum Action {
    Show(u64, String, Vec<String>),
    Hide(u64),
}

/// A popup that records what it was asked to do, with a set of attached channels it can lose.
#[derive(Clone, Default)]
struct FakePopup {
    log: Arc<Mutex<Vec<Action>>>,
    attached: Arc<Mutex<HashSet<u64>>>,
}

impl FakePopup {
    fn attached_with(token: u64) -> Self {
        let popup = Self::default();
        popup.attached.lock().unwrap().insert(token);
        popup
    }
}

impl PopupSink for FakePopup {
    fn show(&mut self, token: u64, context: &str, frame: borno_render::Frame) -> bool {
        if !self.attached(token) {
            return false;
        }
        let texts = frame.rows.iter().map(|row| row.text.clone()).collect();
        self.log
            .lock()
            .unwrap()
            .push(Action::Show(token, context.to_owned(), texts));
        true
    }

    fn hide(&mut self, token: u64) {
        self.log.lock().unwrap().push(Action::Hide(token));
    }

    fn attached(&self, token: u64) -> bool {
        self.attached.lock().unwrap().contains(&token)
    }
}

#[derive(Clone, Default)]
struct FakeWindow {
    shown: Arc<Mutex<usize>>,
}

impl PanelSink for FakeWindow {
    fn show(&mut self, _: borno_render::Frame, _: CursorRect) -> bool {
        *self.shown.lock().unwrap() += 1;
        true
    }

    fn hide(&mut self) {}
}

const CURSOR: CursorRect = CursorRect {
    x: 120.0,
    y: 80.0,
    width: 2.0,
    height: 18.0,
    surface_relative: false,
};

fn rig(popup: &FakePopup, window: Option<&FakeWindow>, token: Option<u64>) -> Rig {
    let words =
        FixedWordSource::new([("kotha".to_owned(), vec!["কথা".to_owned(), "কোথা".to_owned()])]);
    let shared = Shared::new(Arc::new(words), None, None);
    Rig::with_surfaces(
        Config::default(),
        shared,
        window.map(|window| Box::new(window.clone()) as Box<dyn PanelSink>),
        Some(Box::new(popup.clone())),
        token,
    )
}

#[test]
fn a_word_is_shown_in_the_popup_without_any_cursor_and_the_shell_is_told_not_to_draw() {
    let popup = FakePopup::attached_with(TOKEN);
    let mut rig = rig(&popup, None, Some(TOKEN));
    // No cursor was ever reported: the compositor places the popup, not the Server.
    rig.type_text("kotha");
    let reply = rig.function(DOWN);
    assert!(reply.frame.native);
    let log = popup.log.lock().unwrap();
    let Some(Action::Show(token, context, texts)) = log.last() else {
        panic!("expected a Show, got {log:?}");
    };
    assert_eq!((*token, context.as_str()), (TOKEN, "ctx"));
    assert!(texts.iter().any(|text| text == "কথা"));
}

#[test]
fn the_popup_wins_over_the_window_and_the_window_stays_unused() {
    let popup = FakePopup::attached_with(TOKEN);
    let window = FakeWindow::default();
    let mut rig = rig(&popup, Some(&window), Some(TOKEN));
    rig.event(Event::Cursor(CURSOR));
    assert!(rig.type_char('k').frame.native);
    assert_eq!(*window.shown.lock().unwrap(), 0);
    assert!(!popup.log.lock().unwrap().is_empty());
}

#[test]
fn a_context_without_a_popup_uses_the_window_as_before() {
    let popup = FakePopup::attached_with(TOKEN);
    let window = FakeWindow::default();
    let mut rig = rig(&popup, Some(&window), None);
    rig.event(Event::Cursor(CURSOR));
    assert!(rig.type_char('k').frame.native);
    assert_eq!(*window.shown.lock().unwrap(), 1);
    assert!(popup.log.lock().unwrap().is_empty());
}

#[test]
fn a_popup_channel_that_is_not_attached_leaves_the_drawing_to_the_shell() {
    let popup = FakePopup::default();
    let mut rig = rig(&popup, None, Some(TOKEN));
    let reply = rig.type_char('k');
    assert!(!reply.frame.native);
    assert!(popup.log.lock().unwrap().is_empty());
}

#[test]
fn a_lost_channel_falls_back_to_the_window_and_then_the_shell() {
    let popup = FakePopup::attached_with(TOKEN);
    let window = FakeWindow::default();
    let mut rig = rig(&popup, Some(&window), Some(TOKEN));
    rig.event(Event::Cursor(CURSOR));
    assert!(rig.type_char('k').frame.native);
    popup.attached.lock().unwrap().clear();
    // The window takes over because the cursor is known, and the popup is told to hide what it showed.
    assert!(rig.type_char('o').frame.native);
    assert_eq!(*window.shown.lock().unwrap(), 1);
    assert_eq!(popup.log.lock().unwrap().last(), Some(&Action::Hide(TOKEN)));
}

#[test]
fn committing_hides_the_popup() {
    let popup = FakePopup::attached_with(TOKEN);
    let mut rig = rig(&popup, None, Some(TOKEN));
    rig.type_text("kotha");
    let reply = rig.type_char(' ');
    assert!(!reply.frame.native);
    assert_eq!(popup.log.lock().unwrap().last(), Some(&Action::Hide(TOKEN)));
}

#[test]
fn closing_the_session_that_owns_the_popup_hides_it() {
    let popup = FakePopup::attached_with(TOKEN);
    let mut rig = rig(&popup, None, Some(TOKEN));
    rig.type_text("ko");
    rig.router.handle(ClientMessage::Close { session: SESSION });
    assert_eq!(popup.log.lock().unwrap().last(), Some(&Action::Hide(TOKEN)));
}
