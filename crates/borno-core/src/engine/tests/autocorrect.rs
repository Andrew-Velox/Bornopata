//! Corrected spellings in candidate lists and commits.

use std::collections::HashMap;

use borno_transliteration::FixedWordSource;

use super::support::{MemoryLearner, fixed_engine};
use crate::{AutocorrectSource, CandidateSource, Engine};

struct Table(HashMap<&'static str, &'static str>);

impl AutocorrectSource for Table {
    fn lookup(&self, input: &str) -> Vec<String> {
        self.0
            .get(input)
            .map(|text| (*text).to_owned())
            .into_iter()
            .collect()
    }
}

fn table() -> Box<dyn AutocorrectSource> {
    Box::new(Table(HashMap::from([
        ("facebook", "ফেসবুক"),
        ("man", "ম্যান"),
    ])))
}

#[test]
fn a_corrected_spelling_comes_first_and_the_conversion_stays_behind_it() {
    let engine = fixed_engine(&[("facebook", &["ফাচেবুক"])]).with_autocorrect(table());
    let found = engine.query("facebook").candidates;
    assert_eq!(found[0].text, "ফেসবুক");
    assert_eq!(found[0].source, CandidateSource::Autocorrect);
    assert!(found.iter().any(|c| c.text == "ফাচেবুক"));
    assert!(
        engine
            .query("facebooks")
            .candidates
            .iter()
            .all(|c| c.source != CandidateSource::Autocorrect),
        "only the input the table knows is corrected"
    );
}

#[test]
fn a_correction_the_word_list_also_has_is_listed_once() {
    let engine = fixed_engine(&[("facebook", &["ফেসবুক"])]).with_autocorrect(table());
    let found = engine.query("facebook").candidates;
    assert_eq!(found.iter().filter(|c| c.text == "ফেসবুক").count(), 1);
    assert_eq!(found[0].source, CandidateSource::Autocorrect);
}

#[test]
fn what_the_user_keeps_choosing_beats_the_correction() {
    let mut engine = Engine::new(Box::new(FixedWordSource::new([(
        "man".to_owned(),
        vec!["মান".to_owned()],
    )])))
    .with_autocorrect(table())
    .with_learner(Box::new(MemoryLearner::default()));
    for _ in 0..4 {
        for ch in "man".chars() {
            engine.type_char(ch);
        }
        let at = engine
            .candidates()
            .iter()
            .position(|c| c.text == "মান")
            .unwrap();
        engine.highlight(at);
        assert!(engine.commit().is_some());
    }
    assert_eq!(engine.query("man").candidates[0].text, "মান");
}
