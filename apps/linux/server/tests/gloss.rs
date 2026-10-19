//! Glosses beside the candidates of a frame: the ones the glossary has, none for the rest, none when there is no glossary.

mod support;

use std::path::PathBuf;
use std::sync::Arc;

use borno_gloss::{Glossary, GlossaryBuilder};
use borno_linux_server::Shared;
use borno_platform::Config;
use borno_platform::protocol::{Event, FrameGloss};
use borno_transliteration::FixedWordSource;
use support::*;

fn glossary(name: &str, entries: &[(&str, &str, &str)]) -> (Glossary, PathBuf) {
    let mut builder = GlossaryBuilder::new();
    for (word, pos, text) in entries {
        builder.add(word, pos, text).unwrap();
    }
    let path = std::env::temp_dir().join(format!(
        "borno-server-gloss-{}-{name}.bnd",
        std::process::id()
    ));
    builder
        .write(&path, &borno_format::Metadata::default())
        .unwrap();
    (Glossary::open(&path).unwrap(), path)
}

fn rig(glossary: Option<Glossary>) -> Rig {
    let words = FixedWordSource::new([
        ("kotha".to_owned(), vec!["কথা".to_owned(), "কোথা".to_owned()]),
        ("ami".to_owned(), vec!["আমি".to_owned()]),
        ("bhat".to_owned(), vec!["ভাত".to_owned()]),
    ]);
    let shared = Shared::new(
        Arc::new(words),
        Some(borno_learning::FrequencyLearner::in_memory()),
        None,
    )
    .with_glossary(glossary);
    Rig::with(Config::default(), shared)
}

/// Types `text` and returns the reply to its last key.
fn typed(rig: &mut Rig, text: &str) -> Reply {
    let mut last = None;
    for ch in text.chars() {
        last = Some(rig.type_char(ch));
    }
    last.expect("text is not empty")
}

fn gloss(pos: &str, text: &str) -> Option<FrameGloss> {
    Some(FrameGloss {
        pos: pos.into(),
        text: text.into(),
    })
}

#[test]
fn candidates_the_glossary_knows_carry_their_gloss_and_the_others_none() {
    let (glossary, path) = glossary("known", &[("কথা", "n.", "word, speech")]);
    let mut rig = rig(Some(glossary));
    let reply = typed(&mut rig, "kotha");
    let by_text = |text: &str| {
        reply
            .frame
            .candidates
            .iter()
            .find(|c| c.text == text)
            .unwrap()
    };
    assert_eq!(by_text("কথা").gloss, gloss("n.", "word, speech"));
    assert_eq!(by_text("কোথা").gloss, None);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn without_a_glossary_no_candidate_has_a_gloss() {
    let mut rig = rig(None);
    let reply = typed(&mut rig, "kotha");
    assert!(!reply.frame.candidates.is_empty());
    assert!(reply.frame.candidates.iter().all(|c| c.gloss.is_none()));
}

#[test]
fn glosses_do_not_change_the_order_or_the_choice_of_candidates() {
    let (glossary, path) = glossary("order", &[("কোথা", "adv.", "where")]);
    let mut with = rig(Some(glossary));
    let mut without = rig(None);
    let texts = |reply: &Reply| {
        reply
            .frame
            .candidates
            .iter()
            .map(|c| c.text.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        texts(&typed(&mut with, "kotha")),
        texts(&typed(&mut without, "kotha"))
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn next_word_suggestions_have_glosses_too() {
    let (glossary, path) = glossary("next", &[("ভাত", "n.", "rice")]);
    let mut rig = rig(Some(glossary));
    for _ in 0..2 {
        rig.type_text("ami bhat ");
        rig.event(Event::Reset);
    }
    rig.type_text("ami");
    let reply = rig.type_char(' ');
    assert!(reply.frame.suggesting);
    assert_eq!(reply.frame.candidates[0].gloss, gloss("n.", "rice"));
    std::fs::remove_file(path).unwrap();
}
