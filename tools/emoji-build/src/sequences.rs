//! The emoji sequences of Unicode's `emoji-test.txt`, in the order Unicode lists them.

use crate::error::BuildError;

/// Skin tone modifiers: sequences with one are variants of a base emoji and are left out.
const SKIN_TONES: std::ops::RangeInclusive<char> = '\u{1f3fb}'..='\u{1f3ff}';

/// The fully-qualified sequences (the form with U+FE0F where an application needs it), skipping components, skin tone
/// variants and emoji newer than `max_version`, such as `(15, 1)`.
pub fn read(text: &str, max_version: (u32, u32)) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let Some((codes, rest)) = line.split_once(';') else {
            continue;
        };
        let Some((status, comment)) = rest.split_once('#') else {
            continue;
        };
        if status.trim() != "fully-qualified" {
            continue;
        }
        // The comment reads `😀 E1.0 grinning face`.
        let mut words = comment.split_whitespace();
        let (Some(_), Some(version)) = (words.next(), words.next()) else {
            continue;
        };
        if parse_version(version.trim_start_matches('E')).is_none_or(|v| v > max_version) {
            continue;
        }
        let sequence: Option<String> = codes
            .split_whitespace()
            .map(|code| u32::from_str_radix(code, 16).ok().and_then(char::from_u32))
            .collect();
        if let Some(sequence) = sequence
            && !sequence.chars().any(|ch| SKIN_TONES.contains(&ch))
        {
            found.push(sequence);
        }
    }
    found
}

/// `"15.1"` as `(15, 1)`.
pub fn parse_version(text: &str) -> Option<(u32, u32)> {
    let (major, minor) = text.split_once('.')?;
    Some((major.parse().ok()?, minor.parse().ok()?))
}

pub fn parse_version_arg(text: &str) -> Result<(u32, u32), BuildError> {
    parse_version(text).ok_or_else(|| BuildError::Version(text.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_fully_qualified_sequences_of_old_enough_emoji_without_skin_tones() {
        let text = "# comment\n\n1F600 ; fully-qualified # 😀 E1.0 grinning face\n\
                    2764 FE0F ; fully-qualified # ❤️ E0.6 red heart\n\
                    2764 ; unqualified # ❤ E0.6 red heart\n\
                    1F44B 1F3FB ; fully-qualified # 👋🏻 E1.0 waving hand: light skin tone\n\
                    1FAE9 ; fully-qualified # 🫩 E16.0 face with bags under eyes\n\
                    1F3FB ; component # 🏻 E1.0 light skin tone\n";
        assert_eq!(read(text, (15, 1)), ["😀", "❤\u{fe0f}"]);
    }

    #[test]
    fn versions_parse() {
        assert_eq!(parse_version("15.1"), Some((15, 1)));
        assert_eq!(parse_version("x"), None);
        assert!(parse_version_arg("15").is_err());
    }
}
