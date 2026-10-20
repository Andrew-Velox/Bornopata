//! The user's phrase file through the Router.

mod support;

use std::sync::Arc;

use borno_learning::FrequencyLearner;
use borno_linux_server::Shared;
use borno_linux_server::shared::FilePhrases;
use borno_platform::Config;
use borno_platform::protocol::Source;
use borno_transliteration::UpodeshWordSource;
use support::*;

fn rig_with_phrases(name: &str, file: &str) -> (Rig, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("borno-phrases-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("phrases.tsv");
    std::fs::write(&path, file).unwrap();
    let shared = Shared::new(
        Arc::new(UpodeshWordSource::new()),
        Some(FrequencyLearner::in_memory()),
        None,
    )
    .with_phrases(FilePhrases::open(&path));
    (Rig::with(Config::default(), shared), dir)
}

#[test]
fn a_phrase_is_the_first_candidate_and_space_commits_it() {
    let (mut rig, dir) =
        rig_with_phrases("first", "mymail\tme@example.com\nsig\tধন্যবাদ,\\nমোহাব্বত\n");
    let mut reply = None;
    for ch in "mymail".chars() {
        reply = Some(rig.type_char(ch));
    }
    let reply = reply.unwrap();
    assert_eq!(reply.frame.candidates[0].text, "me@example.com");
    assert_eq!(reply.frame.candidates[0].source, Source::Learned);
    assert_eq!(
        rig.type_char(' ').commit.as_deref(),
        Some("me@example.com ")
    );

    rig.type_text("sig");
    assert_eq!(
        rig.type_char(' ').commit.as_deref(),
        Some("ধন্যবাদ,\nমোহাব্বত ")
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn ordinary_words_are_not_touched_by_the_phrases() {
    let (mut rig, dir) = rig_with_phrases("words", "mymail\tme@example.com\n");
    let mut reply = None;
    for ch in "ami".chars() {
        reply = Some(rig.type_char(ch));
    }
    let reply = reply.unwrap();
    assert_eq!(reply.frame.candidates[0].text, "আমি");
    std::fs::remove_dir_all(dir).unwrap();
}
