//! The Server starts over when its config file changes, but never in the middle of a word.

mod support;

use support::*;

fn config_in(name: &str, text: &str) -> std::path::PathBuf {
    let directory =
        std::env::temp_dir().join(format!("borno-reload-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("config.toml");
    std::fs::write(&path, text).unwrap();
    path
}

#[test]
fn an_untouched_config_never_asks_for_a_restart() {
    let path = config_in("untouched", "[general]\nlearning = true\n");
    let mut rig = Rig::scaffold();
    rig.router.watch_config(path);
    rig.router.tick();
    assert!(!rig.router.reload_due());
}

#[test]
fn a_changed_config_asks_for_a_restart() {
    let path = config_in("changed", "[general]\nlearning = true\n");
    let mut rig = Rig::scaffold();
    rig.router.watch_config(path.clone());
    std::fs::write(&path, "[general]\nlearning = false\nemoji = false\n").unwrap();
    rig.router.tick();
    assert!(rig.router.reload_due());
}

#[test]
fn it_waits_for_the_word_to_end() {
    let path = config_in("word", "[general]\nlearning = true\n");
    let mut rig = Rig::scaffold();
    rig.router.watch_config(path.clone());
    rig.type_text("ami");
    std::fs::write(&path, "[general]\nlearning = false\nemoji = false\n").unwrap();
    rig.router.tick();
    assert!(!rig.router.reload_due(), "a word is being typed");
    rig.function(ESCAPE);
    assert!(rig.router.reload_due(), "the word is over");
}

#[test]
fn a_broken_config_is_ignored_until_it_is_fixed() {
    let path = config_in("broken", "[general]\nlearning = true\n");
    let mut rig = Rig::scaffold();
    rig.router.watch_config(path.clone());
    std::fs::write(&path, "[general\nlearning = false\n").unwrap();
    rig.router.tick();
    assert!(!rig.router.reload_due());
    std::fs::write(&path, "[general]\nlearning = false\nemoji = false\n").unwrap();
    rig.router.tick();
    assert!(rig.router.reload_due());
}

#[test]
fn the_forget_marker_empties_what_was_learned_and_goes_away() {
    use borno_platform::protocol::{Event, Source};
    let marker = config_in("forget", "").with_file_name("forget-learned");
    let mut rig = Rig::scaffold();
    rig.router.watch_forget_marker(marker.clone());

    // Choose the second spelling once, so it leads the next time.
    rig.type_text("ami");
    rig.function(DOWN);
    rig.type_char(' ');
    rig.event(Event::Reset);
    let mut reply = None;
    for ch in "ami".chars() {
        reply = Some(rig.type_char(ch));
    }
    assert_eq!(reply.unwrap().frame.candidates[0].source, Source::Learned);
    rig.event(Event::Reset);

    std::fs::write(&marker, b"").unwrap();
    rig.router.tick();
    assert!(
        !marker.exists(),
        "the marker is removed once it has been obeyed"
    );
    let mut reply = None;
    for ch in "ami".chars() {
        reply = Some(rig.type_char(ch));
    }
    assert_ne!(reply.unwrap().frame.candidates[0].source, Source::Learned);
}
