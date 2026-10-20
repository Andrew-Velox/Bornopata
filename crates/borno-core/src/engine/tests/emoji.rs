//! Emoji beside the words in candidate lists.

use super::support::{FixedEmoji, scaffold_engine};
use crate::{CandidateSource, Engine};

fn engine() -> Engine {
    scaffold_engine().with_emoji(Box::new(FixedEmoji(&[
        ("আমি", &["🙋", "👤", "🧍"]),
        ("heart", &["❤️"]),
        ("ami", &["🅰️"]),
    ])))
}

fn texts(engine: &Engine, input: &str) -> Vec<(String, CandidateSource)> {
    engine
        .query(input)
        .candidates
        .into_iter()
        .map(|c| (c.text, c.source))
        .collect()
}

#[test]
fn the_emoji_of_a_word_follows_it_and_at_most_two_come() {
    let found = texts(&engine(), "ami");
    assert_eq!(found[0], ("আমি".to_owned(), CandidateSource::Direct));
    assert_eq!(found[1], ("🙋".to_owned(), CandidateSource::Emoji));
    assert_eq!(found[2], ("👤".to_owned(), CandidateSource::Emoji));
    assert!(!found.iter().any(|(text, _)| text == "🧍"));
}

#[test]
fn the_english_word_typed_gets_its_emoji_after_the_first_word() {
    let found = texts(&engine(), "heart");
    assert!(
        found
            .iter()
            .any(|(text, source)| text == "❤️" && *source == CandidateSource::Emoji)
    );
    assert_ne!(found[0].1, CandidateSource::Emoji);
}

#[test]
fn a_short_roman_word_is_not_looked_up_in_english() {
    let engine = scaffold_engine().with_emoji(Box::new(FixedEmoji(&[("ab", &["🆎"])])));
    assert!(
        texts(&engine, "ab")
            .iter()
            .all(|(_, s)| *s != CandidateSource::Emoji)
    );
}

#[test]
fn a_later_spelling_is_tried_only_when_the_best_word_has_no_emoji() {
    let engine = scaffold_engine().with_emoji(Box::new(FixedEmoji(&[("অমি", &["3️⃣", "4️⃣"])])));
    let found = texts(&engine, "ami");
    let at = found.iter().position(|(text, _)| text == "অমি").unwrap();
    assert_eq!(found[at + 1].0, "3️⃣");

    let engine = scaffold_engine().with_emoji(Box::new(FixedEmoji(&[
        ("আমি", &["1️⃣"]),
        ("অমি", &["3️⃣", "4️⃣"]),
    ])));
    let found = texts(&engine, "ami");
    assert!(found.iter().all(|(text, _)| text != "3️⃣"));
}

#[test]
fn no_more_than_three_emoji_come_and_the_words_stay() {
    let engine = scaffold_engine().with_emoji(Box::new(FixedEmoji(&[
        ("আমি", &["1️⃣", "2️⃣"]),
        ("ami", &["3️⃣", "4️⃣"]),
    ])));
    let found = texts(&engine, "ami");
    let emoji = found.iter().filter(|(_, s)| *s == CandidateSource::Emoji);
    assert_eq!(emoji.count(), 3);
    assert!(found.iter().any(|(text, _)| text == "অমি"));
}

#[test]
fn an_emoji_has_its_name_as_annotation() {
    let mut engine = engine();
    for ch in "ami".chars() {
        engine.type_char(ch);
    }
    let annotations = engine.annotations();
    assert_eq!(annotations[1].as_ref().unwrap().text, "name of 🙋");
    assert!(annotations[0].is_none());
}

#[test]
fn committing_an_emoji_teaches_it_for_the_input() {
    let mut engine = engine().with_learner(Box::new(super::support::MemoryLearner::default()));
    for ch in "ami".chars() {
        engine.type_char(ch);
    }
    engine.highlight(1);
    assert_eq!(engine.commit().as_deref(), Some("🙋"));
    let again = engine.query("ami").candidates;
    assert_eq!(again[0].text, "🙋");
    assert_eq!(again[0].source, CandidateSource::Learned);
}
