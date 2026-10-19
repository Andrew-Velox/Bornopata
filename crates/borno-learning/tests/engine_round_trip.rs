//! A user's selection survives a restart: choose a candidate, flush, reopen, and it leads.

use borno_core::{CandidateSource, Engine};
use borno_learning::FrequencyLearner;
use borno_transliteration::UpodeshWordSource;

fn engine_in(directory: &std::path::Path) -> Engine {
    Engine::new(Box::new(UpodeshWordSource::new()))
        .with_learner(Box::new(FrequencyLearner::open(directory).unwrap()))
}

fn type_word(engine: &mut Engine, roman: &str) {
    for ch in roman.chars() {
        engine.type_char(ch);
    }
}

#[test]
fn a_selection_is_learned_and_survives_a_restart() {
    let directory = std::env::temp_dir().join(format!("borno-round-trip-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);

    let mut engine = engine_in(&directory);
    type_word(&mut engine, "kotha");
    assert_eq!(engine.candidates()[0].text, "কথা");
    engine.highlight(1);
    let chosen = engine.candidates()[1].text.clone();
    engine.commit();
    engine.flush().unwrap();

    let mut restarted = engine_in(&directory);
    type_word(&mut restarted, "kotha");
    assert_eq!(restarted.candidates()[0].text, chosen);
    assert_eq!(restarted.candidates()[0].source, CandidateSource::Learned);

    // Other inputs are untouched by the choice.
    restarted.reset();
    type_word(&mut restarted, "ami");
    assert_eq!(restarted.candidates()[0].text, "আমি");
    std::fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn next_words_come_from_what_the_user_typed() {
    let directory = std::env::temp_dir().join(format!("borno-next-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);

    let mut engine = engine_in(&directory);
    for ch in "ami bhat ".chars() {
        engine.type_char(ch);
    }
    for ch in "ami bhat ".chars() {
        engine.type_char(ch);
    }
    engine.forget_context();
    for ch in "ami ".chars() {
        engine.type_char(ch);
    }
    assert_eq!(engine.next_words(3).first().map(String::as_str), Some("ভাত"));
    let _ = std::fs::remove_dir_all(&directory);
}
