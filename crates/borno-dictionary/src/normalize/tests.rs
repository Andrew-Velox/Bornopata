//! Tokenizing and normalization.

use super::*;

#[test]
fn tokens_split_on_everything_that_is_not_bengali() {
    let found: Vec<&str> = tokens("আমি, tumi 2026। বাংলা-দেশ \"কথা\"").collect();
    assert_eq!(found, ["আমি", "বাংলা", "দেশ", "কথা"]);
}

#[test]
fn nfc_alone_would_split_the_nukta_letters_so_they_are_precomposed() {
    assert_eq!(normalize("বাড\u{09BC}ি").unwrap(), "বা\u{09DC}ি");
    assert_eq!(normalize("বা\u{09DC}ি").unwrap(), "বা\u{09DC}ি");
    assert_eq!(normalize("আষাঢ\u{09BC}").unwrap(), "আষা\u{09DD}");
    assert_eq!(normalize("য\u{09BC}া").unwrap(), "\u{09DF}া");
}

#[test]
fn split_vowel_signs_are_composed() {
    assert_eq!(normalize("কে\u{09BE}থা").unwrap(), "কোথা");
}

#[test]
fn zero_width_joiners_are_removed_except_inside_ryo() {
    assert_eq!(normalize("ক\u{200C}থা").unwrap(), "কথা");
    assert_eq!(normalize("ক\u{200D}থা").unwrap(), "কথা");
    assert_eq!(normalize("র\u{200D}্যাব").unwrap(), "র\u{200D}্যাব");
    assert_eq!(normalize("ক্\u{200C}ষ").unwrap(), "ক্ষ");
}

#[test]
fn well_formed_words_pass() {
    for word in [
        "আমি",
        "বাংলাদেশ",
        "শিক্ষা",
        "বিশ্ববিদ্যাল\u{09DF}",
        "সঙ্গে",
        "চাঁদ",
        "দুঃখ",
        "ও",
        "এ",
        "ক্ষ",
        "আর্থ",
    ] {
        assert_eq!(normalize(word).as_deref(), Some(word), "{word}");
    }
}

#[test]
fn malformed_words_are_rejected() {
    // A sign with nothing to attach to, a dangling or doubled hasanta, a sign after a sign, digits.
    for word in ["ািম", "ক্", "ক্্ষ", "কাা", "্ক", "ঁক", "ক২", "", "ঃ", "কা্ষ", "অা"]
    {
        assert_eq!(normalize(word), None, "{word:?}");
    }
}

#[test]
fn absurdly_long_runs_are_rejected() {
    assert_eq!(normalize(&"ক".repeat(41)), None);
    assert!(normalize(&"ক".repeat(40)).is_some());
}
