//! Annotations beside candidates: aligned with them, optional, and never part of ranking.

use std::collections::HashMap;

use super::support::fixed_engine;
use crate::{Annotation, Annotator};

struct Table(HashMap<&'static str, (&'static str, &'static str)>);

impl Annotator for Table {
    fn annotate(&self, word: &str) -> Option<Annotation> {
        let (pos, text) = self.0.get(word)?;
        Some(Annotation {
            pos: (*pos).to_owned(),
            text: (*text).to_owned(),
        })
    }
}

fn kotha() -> crate::Engine {
    fixed_engine(&[("kotha", &["কথা", "কোথা"])])
}

fn type_kotha(engine: &mut crate::Engine) {
    for ch in "kotha".chars() {
        engine.type_char(ch);
    }
}

#[test]
fn each_candidate_has_its_annotation_in_order_and_unknown_words_have_none() {
    let table = Table(HashMap::from([("কথা", ("n.", "word, talk"))]));
    let mut engine = kotha().with_annotator(Box::new(table));
    type_kotha(&mut engine);
    let annotations = engine.annotations();
    assert_eq!(annotations.len(), engine.candidates().len());
    for (candidate, annotation) in engine.candidates().iter().zip(&annotations) {
        match candidate.text.as_str() {
            "কথা" => assert_eq!(
                annotation,
                &Some(Annotation {
                    pos: "n.".into(),
                    text: "word, talk".into()
                })
            ),
            _ => assert_eq!(annotation, &None),
        }
    }
}

#[test]
fn without_an_annotator_there_is_nothing_to_show() {
    let mut engine = kotha();
    type_kotha(&mut engine);
    assert!(engine.annotations().iter().all(Option::is_none));
    assert!(!engine.candidates().is_empty());
}

#[test]
fn an_annotator_does_not_change_which_candidates_come_or_their_order() {
    let table = Table(HashMap::from([("কোথা", ("adv.", "where"))]));
    let mut plain = kotha();
    let mut annotated = kotha().with_annotator(Box::new(table));
    type_kotha(&mut plain);
    type_kotha(&mut annotated);
    assert_eq!(plain.candidates(), annotated.candidates());
}
