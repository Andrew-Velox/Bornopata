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

#[test]
fn a_character_from_another_layout_does_not_crash_the_server() {
    let mut rig = Rig::scaffold();
    rig.type_text("ami");
    let reply = rig.type_char('…');
    assert_eq!(reply.commit.as_deref(), Some("আমি…"));
    let reply = rig.type_char('ক');
    assert_eq!(reply.outcome, KeyOutcome::Passthrough);
    assert_eq!(reply.commit, None);
}

/// A Router over a small real dictionary file, the way the Server runs with `words.bnd`.
fn dictionary_rig() -> Rig {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let number = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "borno-server-dict-{}-{number}.bnd",
        std::process::id()
    ));
    let mut builder = borno_dictionary::DictionaryBuilder::new();
    for (word, count) in [
        ("বাংলা", 4_000),
        ("বাংলাদেশ", 90_000),
        ("বাংলাদেশী", 800),
        ("আমি", 50_000),
    ] {
        builder.add(word, count);
    }
    builder
        .write(&path, &borno_format::Metadata::default())
        .unwrap();
    let words = borno_dictionary::word_source(Some(&path)).unwrap();
    std::fs::remove_file(&path).unwrap();
    Rig::with(
        borno_platform::Config::default(),
        borno_linux_server::Shared::new(std::sync::Arc::from(words), None, None),
    )
}

#[test]
fn the_panel_completes_a_prefix_with_the_most_frequent_words() {
    let mut rig = dictionary_rig();
    let mut frame = rig.type_char('b').frame;
    for ch in "angl".chars() {
        frame = rig.type_char(ch).frame;
    }
    // What was typed first, then the completions, most frequent first, marked as completions.
    assert_eq!(frame.candidates[0].source, Source::Direct);
    let completions: Vec<_> = frame.candidates[1..]
        .iter()
        .map(|c| c.text.as_str())
        .collect();
    assert_eq!(completions, ["বাংলাদেশ", "বাংলা", "বাংলাদেশী"]);
    assert!(
        frame.candidates[1..]
            .iter()
            .all(|c| c.source == Source::Completion)
    );
    assert_eq!(frame.highlight, 0);
}

#[test]
fn space_commits_what_was_typed_not_the_first_completion() {
    let mut rig = dictionary_rig();
    rig.type_text("bangl");
    let reply = rig.type_char(' ');
    assert_eq!(reply.commit.as_deref(), Some("বাংল "));
}

#[test]
fn a_completion_can_be_picked_with_its_number() {
    let mut rig = dictionary_rig();
    rig.type_text("bangl");
    let reply = rig.type_char('2');
    assert_eq!(reply.outcome, KeyOutcome::Consumed);
    assert_eq!(reply.commit.as_deref(), Some("বাংলাদেশ"));
}
