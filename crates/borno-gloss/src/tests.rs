//! Writing a glossary and reading it back, with damaged files.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use borno_core::Annotator;
use borno_format::{Kind, Metadata, Writer};

use super::*;

fn scratch(name: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "borno-gloss-{}-{}-{name}.bnd",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn metadata() -> Metadata {
    Metadata {
        name: "test glossary".into(),
        license: "CC-BY-SA-4.0".into(),
        attribution: "test".into(),
        ..Metadata::default()
    }
}

fn built(entries: &[(&str, &str, &str)]) -> (Glossary, PathBuf) {
    let mut builder = GlossaryBuilder::new();
    for (word, pos, text) in entries {
        builder.add(word, pos, text).unwrap();
    }
    let path = scratch("built");
    builder.write(&path, &metadata()).unwrap();
    (Glossary::open(&path).unwrap(), path)
}

#[test]
fn what_was_written_is_found_with_its_part_of_speech_and_text() {
    let (glossary, path) = built(&[("ভাত", "n.", "rice"), ("যাওয়া", "v.", "go, leave")]);
    assert_eq!(glossary.len(), 2);
    assert_eq!(
        glossary.lookup("ভাত"),
        Some(Gloss {
            pos: "n.",
            text: "rice"
        })
    );
    assert_eq!(glossary.lookup("যাওয়া").unwrap().text, "go, leave");
    assert_eq!(glossary.lookup("পানি"), None);
    assert_eq!(glossary.lookup(""), None);
    assert_eq!(glossary.lookup("abc"), None);
    assert_eq!(glossary.metadata().entries, 2);
    assert_eq!(glossary.metadata().license, "CC-BY-SA-4.0");
    std::fs::remove_file(path).unwrap();
}

#[test]
fn any_spelling_of_a_word_finds_it() {
    // য়, ড় and ঢ় written as letter + nukta are the same words as the precomposed letters.
    let (glossary, path) = built(&[("যায়", "v.", "goes"), ("বড়", "adj.", "big")]);
    assert_eq!(
        glossary.lookup("যা\u{09AF}\u{09BC}").map(|g| g.text),
        Some("goes")
    );
    assert_eq!(
        glossary.lookup("ব\u{09A1}\u{09BC}").map(|g| g.text),
        Some("big")
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn a_word_added_twice_keeps_the_last_gloss_and_an_entry_without_pos_is_allowed() {
    let (glossary, path) = built(&[("আমি", "pron.", "me"), ("আমি", "", "I")]);
    assert_eq!(glossary.len(), 1);
    assert_eq!(glossary.lookup("আমি"), Some(Gloss { pos: "", text: "I" }));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn bad_entries_are_refused_when_added() {
    let mut builder = GlossaryBuilder::new();
    assert!(builder.add("rice", "n.", "ভাত").is_err());
    assert!(builder.add("ভাত", "n.", "").is_err());
    assert!(
        builder
            .add("ভাত", "n.", &"x".repeat(MAX_TEXT_BYTES + 1))
            .is_err()
    );
    assert!(builder.add("ভাত", "toolongpos", "rice").is_err());
    assert!(builder.add("ভাত", "n.", "ri\tce").is_err());
    assert!(builder.is_empty());
}

#[test]
fn the_glossary_is_an_annotator() {
    let (glossary, path) = built(&[("ভাত", "n.", "rice")]);
    let annotation = glossary.annotate("ভাত").unwrap();
    assert_eq!(
        (annotation.pos.as_str(), annotation.text.as_str()),
        ("n.", "rice")
    );
    assert!(glossary.annotate("পানি").is_none());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn thousands_of_entries_are_all_found() {
    let words: Vec<String> = (0..5000)
        .map(|n| {
            // Distinct valid words: a consonant, a vowel sign and a second consonant chosen by n.
            let consonants: Vec<char> = ('\u{0995}'..='\u{09B9}').collect();
            let first = consonants[n % consonants.len()];
            let second = consonants[(n / consonants.len()) % consonants.len()];
            let third = consonants[(n / 1000) % consonants.len()];
            format!("{first}\u{09BE}{second}\u{09BF}{third}")
        })
        .collect();
    let mut unique = words.clone();
    unique.sort();
    unique.dedup();
    let mut builder = GlossaryBuilder::new();
    for (n, word) in unique.iter().enumerate() {
        builder.add(word, "n.", &format!("gloss {n}")).unwrap();
    }
    let path = scratch("many");
    builder.write(&path, &metadata()).unwrap();
    let glossary = Glossary::open(&path).unwrap();
    assert_eq!(glossary.len(), unique.len());
    for (n, word) in unique.iter().enumerate() {
        assert_eq!(
            glossary.lookup(word).unwrap().text,
            format!("gloss {n}"),
            "{word}"
        );
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn a_file_of_another_kind_a_truncated_file_and_a_damaged_index_are_refused() {
    let (_, path) = built(&[("ভাত", "n.", "rice"), ("পানি", "n.", "water")]);
    let good = std::fs::read(&path).unwrap();

    let other = scratch("other");
    Writer::new(Kind::Dictionary, &metadata())
        .unwrap()
        .write_to(&other)
        .unwrap();
    assert!(matches!(
        Glossary::open(&other),
        Err(GlossaryError::Format(_))
    ));

    let cut = scratch("cut");
    std::fs::write(&cut, &good[..good.len() / 2]).unwrap();
    assert!(Glossary::open(&cut).is_err());

    // Flip every byte of the file in turn: it must open and answer, or fail to open, never panic.
    let flipped = scratch("flipped");
    for at in 0..good.len() {
        let mut damaged = good.clone();
        damaged[at] ^= 0xff;
        std::fs::write(&flipped, &damaged).unwrap();
        if let Ok(glossary) = Glossary::open(&flipped) {
            let _ = glossary.lookup("ভাত");
            let _ = glossary.lookup("পানি");
            let _ = glossary.lookup("নেই");
        }
    }
    for file in [path, other, cut, flipped] {
        std::fs::remove_file(file).unwrap();
    }
}
