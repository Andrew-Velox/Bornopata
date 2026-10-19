//! Building, opening and looking up.

use std::path::PathBuf;

use borno_format::Metadata;

use super::Dictionary;
use crate::{DictionaryBuilder, key};

fn scratch(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("borno-dictionary-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    directory.join("words.bnd")
}

fn metadata() -> Metadata {
    Metadata {
        name: "test".to_owned(),
        license: "CC0-1.0".to_owned(),
        ..Metadata::default()
    }
}

fn build(name: &str, words: &[(&str, u32)]) -> (Dictionary, PathBuf) {
    let path = scratch(name);
    let mut builder = DictionaryBuilder::new();
    for (word, count) in words {
        builder.add(word, *count);
    }
    builder.write(&path, &metadata()).unwrap();
    (Dictionary::open(&path).unwrap(), path)
}

const WORDS: &[(&str, u32)] = &[
    ("আমি", 900),
    ("আমার", 800),
    ("আমরা", 700),
    ("কথা", 500),
    ("কোথা", 600),
    ("বাংলা", 400),
    ("বাংলাদেশ", 450),
    ("বাংলাদেশী", 100),
    ("বাঙালি", 300),
];

#[test]
fn round_trips_words_and_metadata() {
    let (dictionary, path) = build("round-trip", WORDS);
    assert_eq!(dictionary.len(), WORDS.len());
    assert_eq!(dictionary.metadata().entries, WORDS.len() as u64);
    assert_eq!(dictionary.metadata().license, "CC0-1.0");
    let found = dictionary.lookup(&key("আমি"), 5);
    assert_eq!(
        (found[0].text, found[0].frequency, found[0].exact),
        ("আমি", 900, true)
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn words_sharing_a_key_come_most_frequent_first() {
    let (dictionary, _) = build("shared-key", WORDS);
    let found: Vec<_> = dictionary
        .lookup(&key("কথা"), 5)
        .into_iter()
        .filter(|m| m.exact)
        .map(|m| m.text)
        .collect();
    assert_eq!(found, ["কোথা", "কথা"]);
}

#[test]
fn completions_follow_the_exact_words_by_frequency() {
    let (dictionary, _) = build("completions", WORDS);
    let found = dictionary.lookup(&key("বাংল"), 5);
    let texts: Vec<_> = found.iter().map(|m| m.text).collect();
    assert_eq!(texts, ["বাংলাদেশ", "বাংলা", "বাংলাদেশী"]);
    assert!(found.iter().all(|m| !m.exact));
    // The exact word is listed once, before its own extensions.
    let found = dictionary.lookup(&key("বাংলা"), 5);
    assert_eq!((found[0].text, found[0].exact), ("বাংলা", true));
    assert_eq!(found.iter().filter(|m| m.text == "বাংলা").count(), 1);
}

#[test]
fn the_limit_applies_to_exact_and_completions_together() {
    let (dictionary, _) = build("limit", WORDS);
    assert_eq!(dictionary.lookup(&key("আ"), 2).len(), 2);
    assert_eq!(dictionary.lookup(&key("আ"), 0).len(), 0);
    assert!(dictionary.lookup("", 5).is_empty());
    assert!(dictionary.lookup("ঝঝঝ", 5).is_empty());
}

#[test]
fn short_prefixes_use_the_precomputed_lists_and_agree_with_a_scan() {
    let mut words = Vec::new();
    let letters = ["ক", "খ", "গ", "ম", "ল", "প"];
    let mut count = 10_000u32;
    for a in letters {
        for b in letters {
            for c in letters {
                for d in ["", "া", "ি"] {
                    words.push((format!("{a}{b}{c}{d}"), count));
                    count -= 7;
                }
            }
        }
    }
    let refs: Vec<(&str, u32)> = words.iter().map(|(w, c)| (w.as_str(), *c)).collect();
    let (dictionary, _) = build("precomputed", &refs);
    for prefix in ["ক", "কম", "কমল", "মপ"] {
        let fast = dictionary.completions(&key(prefix), 10);
        let scanned = dictionary.scan(&key(prefix), 10);
        assert_eq!(fast, scanned, "{prefix}");
        assert_eq!(fast.len(), if prefix.chars().count() < 3 { 10 } else { 3 }, "{prefix}");
        assert!(fast.windows(2).all(|pair| pair[0] < pair[1]));
    }
    // Longer than the precomputed length goes through the scan and still ranks by frequency.
    let found = dictionary.lookup(&key("কমলা"), 3);
    assert!(found.windows(2).all(|pair| pair[0].exact || !pair[1].exact));
}

#[test]
fn counts_of_the_same_word_add_up() {
    let (dictionary, _) = build("merge", &[("আমি", 5), ("আমি", 7), ("তুমি", 1)]);
    assert_eq!(dictionary.len(), 2);
    assert_eq!(dictionary.lookup(&key("আমি"), 1)[0].frequency, 12);
}

#[test]
fn an_empty_dictionary_opens_and_answers_nothing() {
    let (dictionary, _) = build("empty", &[]);
    assert!(dictionary.is_empty());
    assert!(dictionary.lookup(&key("আমি"), 5).is_empty());
}

#[test]
fn a_damaged_file_is_refused_not_trusted() {
    let (_, path) = build("damaged", WORDS);
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.truncate(bytes.len() / 2);
    std::fs::write(&path, &bytes).unwrap();
    assert!(Dictionary::open(&path).is_err());

    let other = scratch("not-a-dictionary");
    std::fs::write(&other, b"not a bnd file at all").unwrap();
    assert!(Dictionary::open(&other).is_err());
}
