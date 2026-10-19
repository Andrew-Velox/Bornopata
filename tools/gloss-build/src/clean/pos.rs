//! Wiktextract part-of-speech names to the abbreviations shown in the candidate window.

/// The abbreviation for a Wiktextract part of speech, or `None` for parts that carry no useful gloss
/// (letters, symbols, affixes, proverbs).
pub fn abbreviation(pos: &str) -> Option<&'static str> {
    Some(match pos {
        "noun" | "name" => "n.",
        "verb" => "v.",
        "adj" => "adj.",
        "adv" => "adv.",
        "pron" => "pron.",
        "postp" => "postp.",
        "conj" => "conj.",
        "num" => "num.",
        "det" => "det.",
        "intj" => "int.",
        "particle" => "part.",
        "classifier" => "clf.",
        "phrase" => "phr.",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_and_function_words_have_abbreviations_and_letters_do_not() {
        assert_eq!(abbreviation("noun"), Some("n."));
        assert_eq!(abbreviation("name"), Some("n."));
        assert_eq!(abbreviation("postp"), Some("postp."));
        assert_eq!(abbreviation("character"), None);
        assert_eq!(abbreviation("suffix"), None);
    }
}
