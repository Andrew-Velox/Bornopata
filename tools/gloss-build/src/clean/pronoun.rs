//! English pronouns in the case a Bengali pronoun form stands for.

/// `item` (a pronoun in the nominative, as in "he") put into the genitive ("his") or the objective ("him").
/// `None` when it is not a pronoun this table knows.
pub fn inflect(item: &str, genitive: bool) -> Option<&'static str> {
    Some(match (item.to_lowercase().as_str(), genitive) {
        ("he", true) => "his",
        ("she", true) => "her",
        ("it", true) => "its",
        ("they", true) => "their",
        ("i", true) => "my",
        ("we", true) => "our",
        ("you", true) => "your",
        ("who", true) => "whose",
        ("he", false) => "him",
        ("she", false) => "her",
        ("they", false) => "them",
        ("i", false) => "me",
        ("we", false) => "us",
        ("it", false) => "it",
        ("you", false) => "you",
        ("who", false) => "whom",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nominative_pronouns_become_possessive_or_objective() {
        assert_eq!(inflect("he", true), Some("his"));
        assert_eq!(inflect("They", true), Some("their"));
        assert_eq!(inflect("she", false), Some("her"));
        assert_eq!(inflect("this", true), None);
    }
}
