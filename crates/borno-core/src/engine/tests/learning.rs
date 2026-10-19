//! Learning, context ranking and next-word suggestions.

use super::support::{FixedModel, MemoryLearner, fixed_engine};
use crate::{CandidateSource, Engine};

fn learned_engine() -> Engine {
    fixed_engine(&[("kotha", &["কথা", "কোথা", "কোঠা"])])
        .with_learner(Box::new(MemoryLearner::default()))
}

fn type_word(engine: &mut Engine, roman: &str) {
    for ch in roman.chars() {
        engine.type_char(ch);
    }
}

#[test]
fn a_chosen_candidate_comes_first_next_time() {
    let mut engine = learned_engine();
    type_word(&mut engine, "kotha");
    engine.highlight(2);
    let chosen = engine.candidates()[2].text.clone();
    engine.commit();

    type_word(&mut engine, "kotha");
    assert_eq!(engine.candidates()[0].text, chosen);
    assert_eq!(engine.candidates()[0].source, CandidateSource::Learned);
}

#[test]
fn a_learned_text_is_offered_even_if_no_source_produces_it() {
    let mut engine = fixed_engine(&[]).with_learner(Box::new(MemoryLearner::default()));
    type_word(&mut engine, "rahim");
    let direct = engine.candidates()[0].text.clone();
    assert_eq!(engine.candidates().len(), 1);
    engine.commit();
    // The learner now answers for "rahim" with the committed text, so it stays on top, marked learned.
    type_word(&mut engine, "rahim");
    assert_eq!(engine.candidates()[0].text, direct);
    assert_eq!(engine.candidates()[0].source, CandidateSource::Learned);
}

#[test]
fn the_language_model_prefers_the_likelier_successor() {
    let model = FixedModel(vec![("আমি", vec!["কোথা", "কথা"])]);
    let mut engine = fixed_engine(&[("ami", &["আমি"]), ("kotha", &["কথা", "কোথা"])])
        .with_language_model(Box::new(model));
    type_word(&mut engine, "ami ");
    type_word(&mut engine, "kotha");
    let texts: Vec<_> = engine
        .candidates()
        .iter()
        .map(|c| c.text.as_str())
        .collect();
    assert_eq!(texts[0], "কোথা");
}

#[test]
fn next_words_merge_personal_followers_and_the_model_without_repeats() {
    let model = FixedModel(vec![("আমি", vec!["ভাত", "যাই"])]);
    let mut engine = fixed_engine(&[("ami", &["আমি"]), ("jai", &["যাই"])])
        .with_learner(Box::new(MemoryLearner::default()))
        .with_language_model(Box::new(model));
    type_word(&mut engine, "ami jai ami ");
    assert_eq!(engine.previous_word(), Some("আমি"));
    assert_eq!(engine.next_words(5), ["যাই", "ভাত"]);
    assert_eq!(engine.next_words(1), ["যাই"]);
}

#[test]
fn next_words_are_empty_without_any_knowledge() {
    let engine = fixed_engine(&[]);
    assert!(engine.next_words(5).is_empty());
}
