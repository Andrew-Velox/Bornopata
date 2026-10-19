//! One Wiktionary gloss string to short meanings: no parentheses, no articles, nothing that is an explanation.

/// Most characters of one meaning; longer ones are explanations, not glosses.
const MAX_ITEM_CHARS: usize = 28;

/// Most words of one meaning.
const MAX_ITEM_WORDS: usize = 4;

/// A meaning starting with one of these describes how the word is used instead of what it means.
const EXPLANATIONS: [&str; 8] = [
    "used ",
    "see ",
    "indicates",
    "denotes",
    "expressing",
    "expresses",
    "refers",
    "particle",
];

/// Which articles and markers to strip, by the kind of word the gloss is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Verb,
    Other,
}

/// The meanings in `gloss`, in order, cleaned and without duplicates.
pub fn items(gloss: &str, kind: Kind) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for part in without_brackets(gloss).split([',', ';']) {
        let Some(item) = clean(part, kind) else {
            continue;
        };
        if !found.iter().any(|seen| seen.eq_ignore_ascii_case(&item)) {
            found.push(item);
        }
    }
    found
}

fn clean(part: &str, kind: Kind) -> Option<String> {
    let mut text = part.split_whitespace().collect::<Vec<_>>().join(" ");
    text = text.trim_end_matches(['.', ':', ' ']).to_owned();
    let lower = text.to_lowercase();
    // A verb's meaning that starts with an article ("an auxiliary verb") is a description, not a verb.
    if kind == Kind::Verb
        && ["a ", "an ", "the "]
            .iter()
            .any(|article| lower.starts_with(article))
    {
        return None;
    }
    let stripped = match kind {
        Kind::Verb => lower.strip_prefix("to ").map(|_| &text[3..]),
        Kind::Other => ["a ", "an ", "the "]
            .iter()
            .find(|article| lower.starts_with(**article))
            .map(|article| &text[article.len()..]),
    };
    if let Some(rest) = stripped {
        text = rest.trim().to_owned();
    }
    let lower = text.to_lowercase();
    let bengali = text.chars().any(|c| ('\u{0980}'..='\u{09FF}').contains(&c));
    let digit = text.chars().any(|c| c.is_ascii_digit());
    let explanation = EXPLANATIONS.iter().any(|start| lower.starts_with(start));
    let too_long =
        text.chars().count() > MAX_ITEM_CHARS || text.split(' ').count() > MAX_ITEM_WORDS;
    (!text.is_empty() && !bengali && !digit && !explanation && !too_long && !species(&text))
        .then_some(text)
}

/// A Latin species name such as "Ficus benghalensis": a genus with a capital, then a lowercase epithet.
fn species(text: &str) -> bool {
    let mut words = text.split(' ');
    match (words.next(), words.next(), words.next()) {
        (Some(genus), Some(epithet), None) => {
            genus.chars().next().is_some_and(char::is_uppercase)
                && genus.chars().skip(1).all(|c| c.is_ascii_lowercase())
                && epithet.chars().all(|c| c.is_ascii_lowercase())
                && genus.is_ascii()
                && epithet.len() > 3
        }
        _ => false,
    }
}

/// `gloss` with everything in parentheses or square brackets removed, nesting included.
fn without_brackets(gloss: &str) -> String {
    let mut depth = 0usize;
    let mut out = String::with_capacity(gloss.len());
    for c in gloss.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nouns(gloss: &str) -> Vec<String> {
        items(gloss, Kind::Other)
    }

    #[test]
    fn a_list_of_meanings_is_split_in_order() {
        assert_eq!(
            nouns("group, team, band, gang"),
            ["group", "team", "band", "gang"]
        );
        assert_eq!(
            nouns("to be, to become; to come into existence"),
            ["to be", "to become", "to come into existence"]
        );
    }

    #[test]
    fn verbs_lose_to_and_nouns_lose_articles() {
        assert_eq!(items("to be, to become", Kind::Verb), ["be", "become"]);
        assert_eq!(
            nouns("a surname, the sun, an apple"),
            ["surname", "sun", "apple"]
        );
        assert_eq!(nouns("to"), ["to"]);
    }

    #[test]
    fn parentheses_and_brackets_are_removed_with_what_is_inside() {
        assert_eq!(nouns("year (unit of time)"), ["year"]);
        assert_eq!(
            nouns("Bangladesh (a country in South Asia, formerly East Pakistan)"),
            ["Bangladesh"]
        );
        assert_eq!(nouns("light [archaic (poetic)]"), ["light"]);
    }

    #[test]
    fn explanations_numbers_and_bengali_script_are_dropped() {
        assert_eq!(
            nouns("he, she, it; 3rd person nominative singular ordinary pronoun"),
            ["he", "she", "it"]
        );
        assert_eq!(
            nouns("used to show disagreement - no, incorrect"),
            ["incorrect"]
        );
        assert_eq!(nouns("same as কুমোর"), Vec::<String>::new());
        assert_eq!(
            nouns("the tenth character in the Bengali abugida"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn species_names_and_verb_descriptions_are_dropped() {
        assert_eq!(nouns("banyan tree, Ficus benghalensis"), ["banyan tree"]);
        assert_eq!(
            nouns("Bay of Bengal, New Delhi"),
            ["Bay of Bengal", "New Delhi"]
        );
        assert_eq!(items("go, an auxiliary verb", Kind::Verb), ["go"]);
        assert_eq!(nouns("with (accompaniment, not instrument)."), ["with"]);
    }

    #[test]
    fn duplicates_ignoring_case_and_trailing_dots_are_merged() {
        assert_eq!(nouns("Light, light. , LIGHT"), ["Light"]);
        assert_eq!(nouns(""), Vec::<String>::new());
    }
}
