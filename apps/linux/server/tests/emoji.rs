//! Emoji beside the words, through the Router.

mod support;

use std::sync::Arc;

use borno_learning::FrequencyLearner;
use borno_linux_server::Shared;
use borno_platform::Config;
use borno_platform::protocol::Frame;
use borno_transliteration::UpodeshWordSource;
use support::*;

fn rig(emoji: bool) -> Rig {
    let shared = Shared::new(
        Arc::new(UpodeshWordSource::new()),
        Some(FrequencyLearner::in_memory()),
        None,
    )
    .with_emoji(emoji);
    Rig::with(Config::default(), shared)
}

fn frame_for(rig: &mut Rig, input: &str) -> Frame {
    let mut reply = None;
    for ch in input.chars() {
        reply = Some(rig.type_char(ch));
    }
    reply.unwrap().frame
}

#[test]
fn an_english_word_gets_its_emoji_with_the_name_beside_it() {
    let frame = frame_for(&mut rig(true), "heart");
    let heart = frame
        .candidates
        .iter()
        .find(|candidate| candidate.text.contains('❤'))
        .expect("the red heart is offered");
    assert!(
        heart
            .gloss
            .as_ref()
            .is_some_and(|gloss| gloss.pos.is_empty() && !gloss.text.is_empty())
    );
    assert_ne!(frame.candidates[0].text, heart.text);
}

#[test]
fn the_config_switches_emoji_off() {
    let frame = frame_for(&mut rig(false), "heart");
    assert!(frame.candidates.iter().all(|c| !c.text.contains('❤')));
}

#[test]
fn space_commits_the_first_candidate_not_an_emoji() {
    let mut rig = rig(true);
    for ch in "heart".chars() {
        rig.type_char(ch);
    }
    let committed = rig.type_char(' ').commit.unwrap();
    assert!(!committed.contains('❤'));
}
