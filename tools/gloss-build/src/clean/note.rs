//! Telling real meanings from grammar notes and from rare senses.

/// Phrases a gloss starts with when it only describes a form of another word.
const NOTE_STARTS: [&str; 16] = [
    "alternative form of",
    "alternative spelling of",
    "alternative form",
    "form of",
    "plural of",
    "genitive of",
    "objective of",
    "locative of",
    "infinitive of",
    "participle of",
    "abbreviation of",
    "contraction of",
    "romanization of",
    "misspelling of",
    "clipping of",
    "short form of",
];

/// Words that, followed later by "of", make a gloss a note ("second/third-person polite simple present of X").
const NOTE_WORDS: [&str; 12] = [
    "form",
    "spelling",
    "infinitive",
    "participle",
    "conjugation",
    "inflection",
    "tense",
    "present",
    "past",
    "third-person",
    "second-person",
    "first-person",
];

/// Whether `gloss` is a note about a form ("chalita infinitive of করা") and not a meaning. A note always points at the
/// Bengali word it is a form of, which is what tells it from "form of government".
pub fn is_form_note(gloss: &str) -> bool {
    if !gloss
        .chars()
        .any(|c| ('\u{0980}'..='\u{09FF}').contains(&c))
    {
        return false;
    }
    let lower = gloss.to_lowercase();
    if NOTE_STARTS.iter().any(|start| lower.starts_with(start)) {
        return true;
    }
    let Some((before, _)) = lower.split_once(" of ") else {
        return false;
    };
    before
        .split(|c: char| !(c.is_alphanumeric() || c == '-'))
        .any(|word| NOTE_WORDS.contains(&word))
}

/// Tags of senses to use only when nothing else is left: they are rare, dated or not for polite text.
const LOW_REGISTER: [&str; 11] = [
    "obsolete",
    "archaic",
    "dated",
    "rare",
    "dialectal",
    "dialect",
    "poetic",
    "slang",
    "vulgar",
    "offensive",
    "derogatory",
];

pub fn is_low_register(tags: &[String]) -> bool {
    tags.iter().any(|tag| LOW_REGISTER.contains(&tag.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_notes_are_recognized_and_meanings_are_not() {
        for note in [
            "chalita infinitive of করা (kora)",
            "second/third-person polite simple present of করা (kora).",
            "genitive of এ (e)",
            "Rarh form of হিসাব (hiśab)",
            "alternative form of কুমোর (kumōr)",
            "perfective participle of করা (kora).",
            "simple past third-person familiar of আছ- (ach-)",
        ] {
            assert!(is_form_note(note), "{note}");
        }
        for meaning in [
            "to do",
            "group, team, band",
            "form of government",
            "person of high rank",
            "the sun",
            "not of this world",
        ] {
            assert!(!is_form_note(meaning), "{meaning}");
        }
    }

    #[test]
    fn rare_senses_are_marked_by_their_tags() {
        let tags = |list: &[&str]| list.iter().map(|tag| (*tag).to_owned()).collect::<Vec<_>>();
        assert!(is_low_register(&tags(&["dated", "formal"])));
        assert!(!is_low_register(&tags(&["formal"])));
        assert!(!is_low_register(&[]));
    }
}
