//! Structural checks on a Bengali word.

use super::chars::{HASANTA, ZWJ, is_consonant, is_modifier, is_vowel_letter, is_vowel_sign};

/// Whether `text` is spelled like a word: it starts with a letter, a vowel sign only follows a consonant,
/// a hasanta joins two consonants, and no sign is doubled.
pub(super) fn is_well_formed(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let Some(&first) = chars.first() else {
        return false;
    };
    if !(is_consonant(first) || is_vowel_letter(first)) {
        return false;
    }
    for (index, &c) in chars.iter().enumerate() {
        let before = index.checked_sub(1).map(|i| chars[i]);
        let after = chars.get(index + 1).copied();
        let valid = if is_consonant(c) || is_vowel_letter(c) {
            true
        } else if is_vowel_sign(c) {
            before.is_some_and(is_consonant)
        } else if is_modifier(c) {
            before.is_some_and(|b| is_consonant(b) || is_vowel_sign(b) || is_vowel_letter(b))
        } else if c == HASANTA {
            // After a consonant, and either followed by a consonant, or by the joiner of র‍্য.
            before.is_some_and(|b| is_consonant(b) || b == ZWJ) && after.is_some_and(is_consonant)
        } else if c == ZWJ {
            before == Some('\u{09B0}') && after == Some(HASANTA)
        } else {
            false
        };
        if !valid {
            return false;
        }
    }
    true
}
