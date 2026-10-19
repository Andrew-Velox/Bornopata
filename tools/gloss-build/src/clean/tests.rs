//! The cleaner on small entries shaped like Kaikki lines.

use std::collections::HashMap;
use std::path::Path;

use borno_dictionary::normalize;

use super::{Cleaner, Resolved, Route};
use crate::kaikki::{Index, Line};
use crate::prefer::Prefer;

/// A Kaikki line: `senses` are `(gloss, tags, form_of lemma)`.
fn line(
    word: &str,
    pos: &str,
    senses: &[(&str, &[&str], Option<&str>)],
    forms: &[(&str, &[&str])],
) -> Line {
    let senses: Vec<serde_json::Value> = senses
        .iter()
        .map(|(gloss, tags, lemma)| {
            let mut sense = serde_json::json!({"glosses": [gloss], "tags": tags});
            if let Some(lemma) = lemma {
                sense["form_of"] = serde_json::json!([{ "word": lemma }]);
            }
            sense
        })
        .collect();
    let forms: Vec<serde_json::Value> = forms
        .iter()
        .map(|(form, tags)| serde_json::json!({"form": form, "tags": tags}))
        .collect();
    serde_json::from_value(
        serde_json::json!({"word": word, "pos": pos, "senses": senses, "forms": forms}),
    )
    .unwrap()
}

fn resolve(lines: Vec<Line>, counts: &[(&str, u64)], prefer: &str, word: &str) -> Option<Resolved> {
    let mut index = Index::default();
    for line in lines {
        index.add(line);
    }
    let counts: HashMap<String, u64> = counts
        .iter()
        .map(|(w, c)| (normalize(w).unwrap(), *c))
        .collect();
    let prefer = Prefer::parse(prefer, Path::new("prefer.tsv")).unwrap();
    Cleaner::new(&index, &prefer, &counts).resolve(&normalize(word).unwrap())
}

fn quick(lines: Vec<Line>, word: &str) -> Option<(String, String)> {
    resolve(lines, &[], "", word).map(|r| (r.pos.to_owned(), r.text))
}

fn own(pos: &str, text: &str) -> Option<(String, String)> {
    Some((pos.to_owned(), text.to_owned()))
}

#[test]
fn a_noun_keeps_its_first_two_meanings() {
    let lines = vec![line(
        "দল",
        "noun",
        &[("group, team, band, gang, clan, party", &[], None)],
        &[],
    )];
    assert_eq!(quick(lines, "দল"), own("n.", "group, team"));
}

#[test]
fn meanings_come_from_several_senses_and_verbs_lose_to() {
    let lines = vec![
        line(
            "যোগ",
            "noun",
            &[
                ("addition", &[], None),
                ("yoga", &[], None),
                ("conjunction", &[], None),
            ],
            &[],
        ),
        line(
            "হয়",
            "verb",
            &[("to be, to become; to come into existence", &[], None)],
            &[],
        ),
    ];
    assert_eq!(quick(lines.clone(), "যোগ"), own("n.", "addition, yoga"));
    assert_eq!(quick(lines, "হয়"), own("v.", "be, become"));
}

#[test]
fn a_gloss_that_would_be_long_keeps_only_its_first_meaning() {
    let lines = vec![line(
        "দল",
        "noun",
        &[("aaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbb", &[], None)],
        &[],
    )];
    assert_eq!(quick(lines, "দল"), own("n.", "aaaaaaaaaaaaaaaaaaaa"));
}

#[test]
fn letters_are_skipped_and_a_preferred_part_of_speech_wins() {
    let lines = vec![
        line(
            "ও",
            "character",
            &[(
                "The tenth character in the Bengali abugida",
                &["letter"],
                None,
            )],
            &[],
        ),
        line(
            "ও",
            "pron",
            &[("he, she, it; 3rd person nominative singular", &[], None)],
            &[],
        ),
        line("ও", "conj", &[("also", &[], None), ("and", &[], None)], &[]),
    ];
    assert_eq!(quick(lines.clone(), "ও"), own("pron.", "he, she"));
    let preferred = resolve(lines, &[], "ও\tconj\n", "ও").unwrap();
    assert_eq!(
        (preferred.pos, preferred.text.as_str()),
        ("conj.", "also, and")
    );
}

#[test]
fn proper_names_come_after_ordinary_entries() {
    let lines = vec![
        line("কুমার", "name", &[("a surname", &[], None)], &[]),
        line(
            "কুমার",
            "noun",
            &[("bachelor", &[], None), ("prince", &[], None)],
            &[],
        ),
    ];
    assert_eq!(quick(lines, "কুমার"), own("n.", "bachelor, prince"));
}

#[test]
fn rare_senses_are_used_only_when_nothing_else_is_left() {
    let mixed = vec![line(
        "আল",
        "noun",
        &[("old word", &["dated"], None), ("ridge", &[], None)],
        &[],
    )];
    assert_eq!(quick(mixed, "আল"), own("n.", "ridge"));
    let only_rare = vec![line("আল", "noun", &[("old word", &["dated"], None)], &[])];
    assert_eq!(quick(only_rare, "আল"), own("n.", "old word"));
}

#[test]
fn a_grammar_note_is_not_a_gloss() {
    let lines = vec![line(
        "করতে",
        "verb",
        &[("চলিত infinitive of করা", &[], None)],
        &[],
    )];
    assert_eq!(quick(lines, "করতে"), None);
}

#[test]
fn a_verb_form_is_glossed_by_its_lemma() {
    let lines = vec![
        line(
            "করেন",
            "verb",
            &[(
                "second/third-person polite simple present of করা",
                &["form-of", "polite"],
                Some("করা"),
            )],
            &[],
        ),
        line("করা", "verb", &[("to do", &[], None)], &[]),
    ];
    let resolved = resolve(lines, &[], "", "করেন").unwrap();
    assert_eq!((resolved.pos, resolved.text.as_str()), ("v.", "do"));
    assert_eq!(resolved.route, Route::Lemma(normalize("করা").unwrap()));
}

#[test]
fn case_forms_say_of_and_in_and_come_from_the_forms_table_too() {
    let lines = vec![
        line(
            "বাংলাদেশের",
            "name",
            &[(
                "genitive of বাংলাদেশ",
                &["form-of", "genitive"],
                Some("বাংলাদেশ"),
            )],
            &[],
        ),
        line(
            "বাংলাদেশ",
            "name",
            &[("Bangladesh (a country in South Asia)", &[], None)],
            &[],
        ),
        line(
            "ভারত",
            "name",
            &[("India", &[], None)],
            &[
                ("ভারতের", &["genitive"]),
                ("ভারতে", &["indefinite", "locative"]),
                ("ভারতকে", &["objective"]),
            ],
        ),
    ];
    assert_eq!(quick(lines.clone(), "বাংলাদেশের"), own("n.", "of Bangladesh"));
    assert_eq!(quick(lines.clone(), "ভারতের"), own("n.", "of India"));
    assert_eq!(quick(lines.clone(), "ভারতে"), own("n.", "in India"));
    assert_eq!(quick(lines, "ভারতকে"), own("n.", "India"));
}

#[test]
fn the_forms_of_an_adjective_are_not_mapped_but_its_spelling_variants_are() {
    let lines = vec![line(
        "বড়",
        "adj",
        &[("big", &[], None)],
        &[("বড়ো", &["alternative"]), ("বড়তর", &["comparative"])],
    )];
    assert_eq!(quick(lines.clone(), "বড়ো"), own("adj.", "big"));
    assert_eq!(quick(lines, "বড়তর"), None);
}

#[test]
fn a_pronoun_form_beats_an_unrelated_noun_with_the_same_spelling() {
    let lines = vec![
        line(
            "তার",
            "pron",
            &[("genitive of সে", &["form-of", "genitive"], Some("সে"))],
            &[],
        ),
        line(
            "তার",
            "noun",
            &[("cord", &[], None), ("wire", &[], None)],
            &[],
        ),
        line(
            "সে",
            "pron",
            &[("he, she; 3rd person nominative", &[], None)],
            &[],
        ),
    ];
    assert_eq!(quick(lines, "তার"), own("pron.", "his, her"));
}

#[test]
fn a_pronoun_in_the_genitive_is_put_into_the_english_case() {
    let lines = vec![
        line(
            "তাদের",
            "pron",
            &[("genitive of তারা", &["form-of", "genitive"], Some("তারা"))],
            &[],
        ),
        line("তারা", "pron", &[("they", &[], None)], &[]),
        line(
            "তাঁর",
            "pron",
            &[("genitive of তিনি", &["form-of", "genitive"], Some("তিনি"))],
            &[],
        ),
        line(
            "তিনি",
            "pron",
            &[("he, she; honorific", &[], None)],
            &[("তাঁকে", &["objective"])],
        ),
        line(
            "এর",
            "pron",
            &[("genitive of এ", &["form-of", "genitive"], Some("এ"))],
            &[],
        ),
        line("এ", "pron", &[("this", &[], None)], &[]),
    ];
    assert_eq!(quick(lines.clone(), "তাদের"), own("pron.", "their"));
    assert_eq!(quick(lines.clone(), "তাঁর"), own("pron.", "his, her"));
    assert_eq!(quick(lines.clone(), "তাঁকে"), own("pron.", "him, her"));
    // A pronoun the table does not know gets "of".
    assert_eq!(quick(lines, "এর"), own("pron.", "of this"));
}

#[test]
fn a_preferred_part_of_speech_can_pick_the_verb_form_over_a_homograph() {
    let lines = vec![
        line("হয়", "conj", &[("either", &[], None)], &[]),
        line(
            "হয়",
            "noun",
            &[("horse", &[], None)],
            &[("হয়ের", &["genitive"])],
        ),
        line(
            "হওয়া",
            "verb",
            &[("to be, to become", &[], None)],
            &[("হয়", &["present", "third-person"])],
        ),
    ];
    assert_eq!(quick(lines.clone(), "হয়"), own("conj.", "either"));
    let preferred = resolve(lines, &[], "হয়\tverb\n", "হয়").unwrap();
    assert_eq!(
        (preferred.pos, preferred.text.as_str()),
        ("v.", "be, become")
    );
    assert_eq!(preferred.route, Route::Lemma(normalize("হওয়া").unwrap()));
}

#[test]
fn a_form_of_two_lemmas_takes_the_likelier_one() {
    let lines = vec![
        line(
            "ফলে",
            "noun",
            &[("fruit", &[], None)],
            &[("ফলের", &["genitive"])],
        ),
        line(
            "ফল",
            "noun",
            &[("result", &[], None)],
            &[("ফলের", &["genitive"])],
        ),
    ];
    let unfrequent = resolve(lines.clone(), &[("ফলে", 5), ("ফল", 1)], "", "ফলের").unwrap();
    assert_eq!(unfrequent.text, "of fruit");
    let frequent = resolve(lines, &[("ফলে", 1), ("ফল", 5)], "", "ফলের").unwrap();
    assert_eq!(frequent.text, "of result");
}

#[test]
fn an_unknown_word_has_no_gloss() {
    assert_eq!(quick(Vec::new(), "দল"), None);
}
