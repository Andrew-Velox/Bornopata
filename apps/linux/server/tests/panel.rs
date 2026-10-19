//! The self-drawn candidate window: when the Router shows a frame in it, when it hides it, and when the shell must
//! draw the candidates instead.

mod support;

use std::sync::{Arc, Mutex};

use borno_linux_server::Shared;
use borno_linux_server::panel::PanelSink;
use borno_platform::Config;
use borno_platform::protocol::{ClientMessage, CursorRect, Event, Frame, KeyOutcome};
use borno_transliteration::FixedWordSource;
use support::*;

#[derive(Debug, PartialEq)]
enum Action {
    Show(Vec<String>, CursorRect),
    Hide,
}

/// A window that records what it was asked to do, and can be told to die.
#[derive(Clone, Default)]
struct FakePanel {
    log: Arc<Mutex<Vec<Action>>>,
    dead: Arc<Mutex<bool>>,
}

impl PanelSink for FakePanel {
    fn show(&mut self, frame: borno_render::Frame, cursor: CursorRect) -> bool {
        if *self.dead.lock().unwrap() {
            return false;
        }
        let texts = frame.rows.iter().map(|row| row.text.clone()).collect();
        self.log.lock().unwrap().push(Action::Show(texts, cursor));
        true
    }

    fn hide(&mut self) {
        self.log.lock().unwrap().push(Action::Hide);
    }
}

fn rig(panel: &FakePanel) -> Rig {
    let words =
        FixedWordSource::new([("kotha".to_owned(), vec!["কথা".to_owned(), "কোথা".to_owned()])]);
    let shared = Shared::new(Arc::new(words), None, None);
    Rig::with_panel(Config::default(), shared, Some(Box::new(panel.clone())))
}

const CURSOR: CursorRect = CursorRect {
    x: 120.0,
    y: 80.0,
    width: 2.0,
    height: 18.0,
    surface_relative: true,
};

fn frame_of(reply: &Reply) -> &Frame {
    &reply.frame
}

#[test]
fn a_word_is_shown_in_the_window_and_the_shell_is_told_not_to_draw() {
    let panel = FakePanel::default();
    let mut rig = rig(&panel);
    rig.event(Event::Cursor(CURSOR));
    rig.type_text("kotha");
    let reply = rig.function(DOWN);
    assert!(frame_of(&reply).native);
    let log = panel.log.lock().unwrap();
    let Some(Action::Show(texts, cursor)) = log.last() else {
        panic!("expected a Show, got {log:?}");
    };
    assert_eq!(*cursor, CURSOR);
    assert_eq!(texts.len(), 2);
    assert!(texts.iter().any(|text| text == "কথা"));
}

#[test]
fn without_a_known_cursor_the_shell_draws() {
    let panel = FakePanel::default();
    let mut rig = rig(&panel);
    let reply = rig.type_char('k');
    assert!(!reply.frame.native);
    assert!(panel.log.lock().unwrap().is_empty());
}

#[test]
fn committing_hides_the_window() {
    let panel = FakePanel::default();
    let mut rig = rig(&panel);
    rig.event(Event::Cursor(CURSOR));
    rig.type_text("kotha");
    let reply = rig.type_char(' ');
    assert!(!reply.frame.native);
    assert_eq!(panel.log.lock().unwrap().last(), Some(&Action::Hide));
}

#[test]
fn a_moved_cursor_moves_the_window_while_a_word_is_shown() {
    let panel = FakePanel::default();
    let mut rig = rig(&panel);
    rig.event(Event::Cursor(CURSOR));
    rig.type_text("ko");
    let moved = CursorRect {
        x: 300.0,
        y: 400.0,
        ..CURSOR
    };
    let reply = rig.event(Event::Cursor(moved));
    assert!(reply.frame.native);
    let log = panel.log.lock().unwrap();
    assert!(matches!(log.last(), Some(Action::Show(_, cursor)) if *cursor == moved));
}

#[test]
fn a_window_that_stopped_hands_the_drawing_back_to_the_shell() {
    let panel = FakePanel::default();
    let mut rig = rig(&panel);
    rig.event(Event::Cursor(CURSOR));
    assert!(rig.type_char('k').frame.native);
    *panel.dead.lock().unwrap() = true;
    let reply = rig.type_char('o');
    assert!(!reply.frame.native);
    // And it stays with the shell afterwards, even if the fake would work again.
    *panel.dead.lock().unwrap() = false;
    assert!(!rig.type_char('t').frame.native);
}

#[test]
fn closing_the_session_that_owns_the_window_hides_it() {
    let panel = FakePanel::default();
    let mut rig = rig(&panel);
    rig.event(Event::Cursor(CURSOR));
    rig.type_text("ko");
    rig.router.handle(ClientMessage::Close { session: SESSION });
    assert_eq!(panel.log.lock().unwrap().last(), Some(&Action::Hide));
}

#[test]
fn next_word_suggestions_show_without_numbers_and_with_the_key_hint() {
    let panel = FakePanel::default();
    let words = FixedWordSource::new([
        ("ami".to_owned(), vec!["আমি".to_owned()]),
        ("bhat".to_owned(), vec!["ভাত".to_owned()]),
    ]);
    let shared = Shared::new(
        Arc::new(words),
        Some(borno_learning::FrequencyLearner::in_memory()),
        None,
    );
    let mut rig = Rig::with_panel(Config::default(), shared, Some(Box::new(panel.clone())));
    rig.event(Event::Cursor(CURSOR));
    for _ in 0..2 {
        rig.type_text("ami bhat ");
        rig.event(Event::Reset);
    }
    rig.type_text("ami");
    let reply = rig.type_char(' ');
    assert!(reply.frame.suggesting && reply.frame.native);
    assert_eq!(reply.outcome, KeyOutcome::Consumed);
    let log = panel.log.lock().unwrap();
    let Some(Action::Show(texts, _)) = log.last() else {
        panic!("expected a Show");
    };
    assert_eq!(texts, &["ভাত".to_owned()]);
}

#[test]
fn an_all_zero_cursor_is_not_a_position_so_the_shell_draws() {
    let panel = FakePanel::default();
    let mut rig = rig(&panel);
    rig.event(Event::Cursor(CursorRect {
        surface_relative: true,
        ..CursorRect::default()
    }));
    rig.type_text("kotha");
    let reply = rig.function(DOWN);
    assert!(!frame_of(&reply).native);
    assert!(
        !panel
            .log
            .lock()
            .unwrap()
            .iter()
            .any(|action| matches!(action, Action::Show(..)))
    );
}
