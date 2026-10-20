//! The user's own phrases in candidate lists and commits.

use borno_transliteration::FixedWordSource;

use super::support::{MemoryLearner, scaffold_engine};
use crate::{CandidateSource, Engine, PhraseBook, PhraseSource};

fn book(text: &str) -> Box<dyn PhraseSource> {
    Box::new(PhraseBook::parse(text).book)
}

#[test]
fn a_phrase_comes_first_for_exactly_its_code() {
    let engine = scaffold_engine().with_phrases(book("ami\tamar nam Mohabbat\nami\tআমি!\n"));
    let found = engine.query("ami").candidates;
    assert_eq!(found[0].text, "amar nam Mohabbat");
    assert_eq!(found[0].source, CandidateSource::Phrase);
    assert_eq!(found[1].text, "আমি!");
    assert!(
        found.iter().any(|c| c.text == "আমি"),
        "the ordinary words stay behind the phrases"
    );
    assert!(
        engine
            .query("amit")
            .candidates
            .iter()
            .all(|c| c.source != CandidateSource::Phrase),
        "a longer input is not the code"
    );
}

#[test]
fn a_phrase_text_the_word_list_also_has_is_still_a_phrase() {
    let engine = scaffold_engine().with_phrases(book("ami\tআমি\n"));
    let found = engine.query("ami").candidates;
    assert_eq!(found[0].text, "আমি");
    assert_eq!(found[0].source, CandidateSource::Phrase);
    assert_eq!(found.iter().filter(|c| c.text == "আমি").count(), 1);
}

#[test]
fn committing_a_phrase_teaches_the_learner_nothing() {
    let mut engine = Engine::new(Box::new(FixedWordSource::new([])))
        .with_phrases(book("mymail\tme@example.com\n"))
        .with_learner(Box::new(MemoryLearner::default()));
    for ch in "mymail".chars() {
        engine.type_char(ch);
    }
    assert_eq!(engine.commit().as_deref(), Some("me@example.com"));
    assert!(engine.learner.choices("mymail").is_empty());
    assert_eq!(engine.learner.weight("me@example.com"), 0);
}
