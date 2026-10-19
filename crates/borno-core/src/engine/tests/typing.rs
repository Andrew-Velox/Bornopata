//! Typing, editing and committing through the engine.

use super::support::scaffold_engine;
use crate::Typed;

fn type_all(engine: &mut crate::Engine, text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if let Typed::Commit(committed) = engine.type_char(ch) {
            out.push_str(&committed);
        }
    }
    out
}

#[test]
fn a_space_commits_the_top_candidate_and_passes_through() {
    let mut engine = scaffold_engine();
    assert_eq!(type_all(&mut engine, "ami bangla "), "আমি বাংলা ");
    assert!(engine.composition().is_empty());
    assert_eq!(engine.previous_word(), Some("বাংলা"));
}

#[test]
fn punctuation_is_converted_and_ends_the_sentence() {
    let mut engine = scaffold_engine();
    assert_eq!(type_all(&mut engine, "ami."), "আমি।");
    assert_eq!(engine.previous_word(), None);
    assert_eq!(type_all(&mut engine, "1"), "১");
}

#[test]
fn typing_keeps_case_and_shows_candidates_while_composing() {
    let mut engine = scaffold_engine();
    assert_eq!(engine.type_char('T'), Typed::Composing);
    engine.type_char('a');
    assert_eq!(engine.composition().as_str(), "Ta");
    assert_eq!(engine.candidates()[0].text, "টা");
}

#[test]
fn backspace_edits_the_word_then_defers_to_the_application() {
    let mut engine = scaffold_engine();
    type_all(&mut engine, "amr");
    assert!(engine.backspace());
    assert_eq!(engine.composition().as_str(), "am");
    assert!(engine.backspace());
    assert!(engine.backspace());
    assert!(!engine.backspace());
    assert!(engine.candidates().is_empty());
}

#[test]
fn highlight_moves_wrap_and_commit_uses_the_highlighted_candidate() {
    let mut engine = scaffold_engine();
    type_all(&mut engine, "ami");
    let count = engine.candidates().len() as isize;
    engine.move_highlight(-1);
    assert_eq!(engine.highlighted(), (count - 1) as usize);
    engine.move_highlight(1);
    assert_eq!(engine.highlighted(), 0);
    engine.highlight(1);
    let second = engine.candidates()[1].text.clone();
    assert_eq!(engine.commit(), Some(second));
    assert_eq!(engine.commit(), None);
}

#[test]
fn reset_drops_the_word_without_committing() {
    let mut engine = scaffold_engine();
    type_all(&mut engine, "ami");
    engine.reset();
    assert!(engine.composition().is_empty());
    assert_eq!(engine.previous_word(), None);
}

#[test]
fn a_mark_after_a_letter_stays_in_the_word() {
    let mut engine = scaffold_engine();
    type_all(&mut engine, "a^");
    assert_eq!(engine.composition().as_str(), "a^");
}
