//! Frequencies and prefix completion in the candidate list.

use super::support::stub_engine;
use crate::CandidateSource;

const WORDS: &[(&str, &str, u32)] = &[
    ("bangla", "বাংলা", 4_000),
    ("bangladesh", "বাংলাদেশ", 90_000),
    ("banglish", "বাংলিশ", 50),
    ("kotha", "কথা", 5_000),
    ("kotha", "কোথা", 80_000),
    ("kotha", "কতা", 10),
];

fn engine() -> crate::Engine {
    stub_engine(WORDS.to_vec())
}

fn texts(engine: &crate::Engine, roman: &str) -> Vec<(String, CandidateSource)> {
    engine
        .query(roman)
        .candidates
        .into_iter()
        .map(|c| (c.text, c.source))
        .collect()
}

#[test]
fn completions_come_after_what_was_typed_even_when_they_are_more_frequent() {
    let found = texts(&engine(), "bangla");
    assert_eq!(found[0], ("বাংলা".to_owned(), CandidateSource::Direct));
    assert_eq!(found[1], ("বাংলাদেশ".to_owned(), CandidateSource::Completion));
    assert_eq!(found.len(), 2);
}

#[test]
fn a_prefix_that_is_no_word_still_shows_its_conversion_first_then_completions_by_frequency() {
    let found = texts(&engine(), "bangl");
    assert_eq!(found[0].1, CandidateSource::Direct);
    let rest: Vec<_> = found[1..].iter().map(|(text, _)| text.as_str()).collect();
    assert_eq!(rest, ["বাংলাদেশ", "বাংলা", "বাংলিশ"]);
    assert!(
        found[1..]
            .iter()
            .all(|(_, source)| *source == CandidateSource::Completion)
    );
}

#[test]
fn spellings_of_the_input_are_ranked_by_frequency() {
    let found = texts(&engine(), "kotha");
    // কোথা is sixteen times as frequent as the direct conversion কথা, which is worth more than typing it exactly.
    assert_eq!(found[0], ("কোথা".to_owned(), CandidateSource::Dictionary));
    assert_eq!(found[1], ("কথা".to_owned(), CandidateSource::Direct));
    assert_eq!(found[2], ("কতা".to_owned(), CandidateSource::Dictionary));
}

#[test]
fn the_direct_conversion_stays_first_when_it_is_about_as_frequent() {
    let engine = stub_engine(vec![("kotha", "কথা", 50_000), ("kotha", "কোথা", 60_000)]);
    let found = texts(&engine, "kotha");
    assert_eq!(found[0], ("কথা".to_owned(), CandidateSource::Direct));
}

#[test]
fn a_space_commits_what_was_typed_not_a_more_frequent_completion() {
    let mut engine = engine();
    for ch in "bangla ".chars() {
        engine.type_char(ch);
    }
    assert_eq!(engine.previous_word(), Some("বাংলা"));
}

#[test]
fn a_completion_can_be_chosen_and_is_learned_for_that_input() {
    let mut engine = engine().with_learner(Box::new(super::support::MemoryLearner::default()));
    for ch in "bangl".chars() {
        engine.type_char(ch);
    }
    engine.highlight(1);
    assert_eq!(engine.commit().as_deref(), Some("বাংলাদেশ"));
    for ch in "bangl".chars() {
        engine.type_char(ch);
    }
    assert_eq!(engine.candidates()[0].text, "বাংলাদেশ");
    assert_eq!(engine.candidates()[0].source, CandidateSource::Learned);
}

#[test]
fn the_candidate_limit_cuts_completions_before_spellings() {
    let engine = stub_engine(WORDS.to_vec()).with_candidate_limit(2);
    let found = texts(&engine, "bangla");
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].1, CandidateSource::Direct);
}
