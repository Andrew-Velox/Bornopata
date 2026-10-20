//! Date and time shortcuts in candidate lists and commits.

use super::support::{FixedClock, MemoryLearner, scaffold_engine};
use crate::{CandidateSource, LocalTime};

const NOON: LocalTime = LocalTime {
    year: 2026,
    month: 10,
    day: 20,
    hour: 12,
    minute: 0,
    weekday: 2,
};

fn engine() -> crate::Engine {
    scaffold_engine().with_clock(Box::new(FixedClock(NOON)))
}

#[test]
fn the_date_follows_the_first_candidate() {
    let found = engine().query("tarikh").candidates;
    assert_ne!(found[0].source, CandidateSource::Shortcut);
    assert_eq!(found[0].text, "তারিখ");
    assert_eq!(found[1].text, "২০ অক্টোবর ২০২৬");
    assert!(
        found[1..5]
            .iter()
            .all(|c| c.source == CandidateSource::Shortcut)
    );
}

#[test]
fn without_a_clock_nothing_is_added() {
    let found = scaffold_engine().query("tarikh").candidates;
    assert!(found.iter().all(|c| c.source != CandidateSource::Shortcut));
}

#[test]
fn other_words_are_left_alone() {
    let found = engine().query("ami").candidates;
    assert!(found.iter().all(|c| c.source != CandidateSource::Shortcut));
}

#[test]
fn the_time_of_day_is_offered_for_somoy() {
    let found = engine().query("somoy").candidates;
    assert!(found.iter().any(|c| c.text == "দুপুর ১২:০০"));
}

#[test]
fn a_date_is_not_learned() {
    let mut engine = engine().with_learner(Box::new(MemoryLearner::default()));
    for ch in "tarikh".chars() {
        engine.type_char(ch);
    }
    engine.highlight(1);
    assert_eq!(engine.commit().as_deref(), Some("২০ অক্টোবর ২০২৬"));
    assert!(engine.learner.choices("tarikh").is_empty());
}
