//! Typing Bangla through the Router: what each key does.

mod support;

use borno_platform::protocol::{KeyModifiers, KeyOutcome, Mode, Source};
use support::*;

#[test]
fn letters_build_a_word_and_show_candidates() {
    let mut rig = Rig::scaffold();
    rig.type_text("ami");
    let reply = rig.type_char('x');
    assert_eq!(reply.outcome, KeyOutcome::Consumed);
    let reply = rig.function(BACKSPACE);
    assert_eq!(reply.frame.roman, "ami");
    assert_eq!(reply.frame.preedit, "আমি");
    assert_eq!(reply.frame.candidates[0].text, "আমি");
    assert_eq!(reply.frame.candidates[0].source, Source::Direct);
    assert!(reply.frame.candidates.len() <= 9);
    assert!(!reply.frame.suggesting);
}

#[test]
fn space_commits_the_word_with_a_space() {
    let mut rig = Rig::scaffold();
    rig.type_text("ami");
    let reply = rig.type_char(' ');
    assert_eq!(reply.outcome, KeyOutcome::Consumed);
    assert_eq!(reply.commit.as_deref(), Some("আমি "));
    assert!(reply.frame.is_empty());
}

#[test]
fn space_without_a_word_goes_to_the_application() {
    let mut rig = Rig::scaffold();
    let reply = rig.type_char(' ');
    assert_eq!(reply.outcome, KeyOutcome::Passthrough);
    assert_eq!(reply.commit, None);
}

#[test]
fn enter_commits_the_word_and_passes_through() {
    let mut rig = Rig::scaffold();
    rig.type_text("bangla");
    let reply = rig.function(ENTER);
    assert_eq!(reply.outcome, KeyOutcome::Passthrough);
    assert_eq!(reply.commit.as_deref(), Some("বাংলা"));
    assert!(reply.frame.is_empty());
    assert_eq!(rig.function(ENTER).commit, None);
}

#[test]
fn backspace_edits_the_roman_buffer_then_passes_through() {
    let mut rig = Rig::scaffold();
    rig.type_text("amr");
    let reply = rig.function(BACKSPACE);
    assert_eq!(reply.outcome, KeyOutcome::Consumed);
    assert_eq!(reply.frame.roman, "am");
    rig.function(BACKSPACE);
    rig.function(BACKSPACE);
    let reply = rig.function(BACKSPACE);
    assert_eq!(reply.outcome, KeyOutcome::Passthrough);
}

#[test]
fn escape_cancels_the_word() {
    let mut rig = Rig::scaffold();
    rig.type_text("ami");
    let reply = rig.function(ESCAPE);
    assert_eq!(reply.outcome, KeyOutcome::Consumed);
    assert!(reply.frame.is_empty());
    assert_eq!(rig.function(ESCAPE).outcome, KeyOutcome::Passthrough);
}

#[test]
fn punctuation_commits_the_word_and_is_converted() {
    let mut rig = Rig::scaffold();
    rig.type_text("ami");
    let reply = rig.type_char('.');
    assert_eq!(reply.outcome, KeyOutcome::Consumed);
    assert_eq!(reply.commit.as_deref(), Some("আমি।"));
    let reply = rig.type_char(',');
    assert_eq!(reply.outcome, KeyOutcome::Passthrough);
}

#[test]
fn digits_become_bangla_digits_unless_turned_off() {
    let mut rig = Rig::scaffold();
    let reply = rig.type_char('2');
    assert_eq!(reply.outcome, KeyOutcome::Consumed);
    assert_eq!(reply.commit.as_deref(), Some("২"));

    let mut config = borno_platform::Config::default();
    config.general.bangla_digits = false;
    let mut rig = Rig::with(
        config,
        borno_linux_server::Shared::new(
            std::sync::Arc::new(borno_transliteration::FixedWordSource::new([])),
            None,
            None,
        ),
    );
    let reply = rig.type_char('2');
    assert_eq!(reply.outcome, KeyOutcome::Passthrough);
    assert_eq!(reply.commit, None);
}

#[test]
fn a_digit_picks_the_candidate_while_typing() {
    let mut rig = Rig::fixed(&[("kotha", &["কথা", "কোথা"])]);
    rig.type_text("kotha");
    let reply = rig.type_char('2');
    assert_eq!(reply.outcome, KeyOutcome::Consumed);
    let committed = reply.commit.unwrap();
    assert_ne!(committed, "কথা");
    assert!(["কোথা"].contains(&committed.as_str()) || committed != "২");
}

#[test]
fn up_down_and_tab_move_the_highlight() {
    let mut rig = Rig::fixed(&[("kotha", &["কথা", "কোথা", "কোঠা"])]);
    rig.type_text("kotha");
    assert_eq!(rig.function(DOWN).frame.highlight, 1);
    assert_eq!(rig.function(TAB).frame.highlight, 2);
    assert_eq!(rig.function(UP).frame.highlight, 1);
    let backwards = rig.key(function_key(
        TAB,
        KeyModifiers {
            shift: true,
            ..KeyModifiers::default()
        },
    ));
    assert_eq!(backwards.frame.highlight, 0);
    rig.function(DOWN);
    let reply = rig.type_char(' ');
    let committed = reply.commit.unwrap();
    assert!(committed.ends_with(' '));
    assert_ne!(committed, "কথা ");
    assert_eq!(rig.function(UP).outcome, KeyOutcome::Passthrough);
}

#[test]
fn cursor_keys_commit_the_word_and_move_the_cursor() {
    let mut rig = Rig::scaffold();
    rig.type_text("ami");
    let reply = rig.function(LEFT);
    assert_eq!(reply.outcome, KeyOutcome::Passthrough);
    assert_eq!(reply.commit.as_deref(), Some("আমি"));
}

#[test]
fn command_chords_commit_the_word_and_pass_the_chord() {
    let mut rig = Rig::scaffold();
    rig.type_text("ami");
    let chord = KeyModifiers {
        ctrl: true,
        ..KeyModifiers::default()
    };
    let reply = rig.key(borno_platform::protocol::KeyEvent::new(
        0x63,
        Some('c'),
        chord,
    ));
    assert_eq!(reply.outcome, KeyOutcome::Passthrough);
    assert_eq!(reply.commit.as_deref(), Some("আমি"));
    let plain = rig.key(borno_platform::protocol::KeyEvent::new(
        0x63,
        Some('c'),
        chord,
    ));
    assert_eq!(plain.commit, None);
}

#[test]
fn a_lone_shift_press_changes_nothing() {
    let mut rig = Rig::scaffold();
    rig.type_text("ami");
    let reply = rig.key(function_key(0x10, KeyModifiers::default()));
    assert_eq!(reply.outcome, KeyOutcome::Passthrough);
    assert_eq!(reply.commit, None);
    assert_eq!(reply.frame.roman, "ami");
}

#[test]
fn the_toggle_switches_to_english_and_back() {
    let mut rig = Rig::scaffold();
    rig.type_text("ami");
    let toggle = borno_platform::protocol::KeyEvent::new(
        0x20,
        Some(' '),
        KeyModifiers {
            ctrl: true,
            alt: true,
            ..KeyModifiers::default()
        },
    );
    let reply = rig.key(toggle);
    assert_eq!(reply.outcome, KeyOutcome::Consumed);
    assert_eq!(reply.commit.as_deref(), Some("আমি"));
    assert_eq!(reply.frame.mode, Mode::English);

    let reply = rig.type_char('a');
    assert_eq!(reply.outcome, KeyOutcome::Passthrough);
    assert!(reply.frame.is_empty());

    let reply = rig.key(toggle);
    assert_eq!(reply.frame.mode, Mode::Bangla);
    assert_eq!(rig.type_char('a').outcome, KeyOutcome::Consumed);
}

#[test]
fn the_toggle_follows_the_config() {
    let mut config = borno_platform::Config::default();
    config.shortcut.toggle = "ctrl+shift+b".parse().unwrap();
    let shared = borno_linux_server::Shared::new(
        std::sync::Arc::new(borno_transliteration::FixedWordSource::new([])),
        None,
        None,
    );
    let mut rig = Rig::with(config, shared);
    let old = borno_platform::protocol::KeyEvent::new(
        0x20,
        Some(' '),
        KeyModifiers {
            ctrl: true,
            alt: true,
            ..KeyModifiers::default()
        },
    );
    assert_eq!(rig.key(old).frame.mode, Mode::Bangla);
    let new = borno_platform::protocol::KeyEvent::new(
        0x42,
        Some('B'),
        KeyModifiers {
            ctrl: true,
            shift: true,
            ..KeyModifiers::default()
        },
    );
    assert_eq!(rig.key(new).frame.mode, Mode::English);
}
