//! Candidate generation for single words.

use super::support::{fixed_engine, scaffold_engine};
use crate::CandidateSource;

fn texts(engine: &crate::Engine, roman: &str) -> Vec<String> {
    engine
        .query(roman)
        .candidates
        .into_iter()
        .map(|candidate| candidate.text)
        .collect()
}

#[test]
fn ami_puts_the_real_word_first() {
    let engine = scaffold_engine();
    let query = engine.query("ami");
    assert_eq!(query.candidates[0].text, "আমি");
    assert_eq!(query.candidates[0].source, CandidateSource::Direct);
    assert!(query.candidates.iter().any(|c| c.text == "অমি"));
}

#[test]
fn bangla_and_kotha_lead_with_the_direct_conversion() {
    let engine = scaffold_engine();
    assert_eq!(texts(&engine, "bangla")[0], "বাংলা");
    assert_eq!(texts(&engine, "kotha")[0], "কথা");
}

#[test]
fn a_conjunct_comes_out_of_the_rules() {
    let engine = scaffold_engine();
    let found = texts(&engine, "shikkha");
    assert_eq!(found[0], "শিক্ষা");
}

#[test]
fn a_dictionary_word_beats_an_unbacked_direct_conversion() {
    let engine = scaffold_engine();
    let query = engine.query("ShikkhA");
    assert_eq!(query.candidates[0].text, "শিক্ষা");
    let direct = query.candidates.iter().find(|c| c.text == "ষিক্ষা").unwrap();
    assert_eq!(direct.source, CandidateSource::Direct);
}

#[test]
fn letter_case_selects_the_letter() {
    let engine = scaffold_engine();
    assert_eq!(texts(&engine, "ta")[0], "তা");
    assert_eq!(texts(&engine, "Ta")[0], "টা");
}

#[test]
fn the_direct_conversion_is_shown_when_nothing_matches() {
    let engine = fixed_engine(&[]);
    let query = engine.query("zyxwv");
    assert_eq!(query.candidates.len(), 1);
    assert_eq!(query.candidates[0].source, CandidateSource::Direct);
}

#[test]
fn empty_input_has_no_candidates() {
    let engine = scaffold_engine();
    assert!(engine.query("").candidates.is_empty());
}

#[test]
fn long_input_still_answers_with_the_direct_conversion() {
    let engine = scaffold_engine();
    let roman = "a".repeat(5000);
    let query = engine.query(&roman);
    assert_eq!(query.candidates.len(), 1);
    assert_eq!(query.candidates[0].text.chars().count(), 5000);
}

#[test]
fn equal_scores_keep_a_stable_order() {
    let engine = fixed_engine(&[("x", &["খ", "ক", "গ"])]);
    let first = texts(&engine, "x");
    assert_eq!(first, texts(&engine, "x"));
}

#[test]
fn the_candidate_limit_keeps_the_best_ones() {
    let engine = fixed_engine(&[("x", &["ক", "খ", "গ", "ঘ"])]).with_candidate_limit(2);
    let all = texts(&fixed_engine(&[("x", &["ক", "খ", "গ", "ঘ"])]), "x");
    assert_eq!(texts(&engine, "x"), all[..2]);
}
