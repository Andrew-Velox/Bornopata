//! Cleaning corpus text into dictionary words: tokenizing, normalizing the Unicode form and rejecting malformed spellings.

mod chars;
mod validate;

#[cfg(test)]
mod tests;

use unicode_normalization::UnicodeNormalization;

use chars::{HASANTA, NUKTA, ZWJ, ZWNJ, is_word_char};

/// Longest word kept, in characters. Longer runs are almost always glued text or markup.
const MAX_WORD_CHARS: usize = 40;

/// The Bengali tokens of `text`, as written: maximal runs of Bengali letters, signs and joiners.
/// Digits, danda, Latin text and punctuation separate tokens.
pub fn tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !is_word_char(c))
        .filter(|token| !token.is_empty())
}

/// The canonical spelling of `token`, or `None` when it is not a well-formed word.
///
/// Canonical means what `okkhor` produces, so a typed word and its dictionary entry are equal strings:
/// NFC, then ড় ঢ় য় precomposed (NFC alone splits them because they are composition exclusions),
/// zero-width non-joiners removed, and the zero-width joiner kept only inside র‍্য.
pub fn normalize(token: &str) -> Option<String> {
    let text = canonical(token);
    if text.is_empty() || text.chars().count() > MAX_WORD_CHARS || !validate::is_well_formed(&text)
    {
        return None;
    }
    Some(text)
}

/// The canonical form without judging the word; shared with the phonetic key.
pub(crate) fn canonical(text: &str) -> String {
    let composed: Vec<char> = text.nfc().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0;
    while index < composed.len() {
        let c = composed[index];
        let next = composed.get(index + 1).copied();
        match (c, next) {
            ('\u{09A1}', Some(NUKTA)) => {
                out.push('\u{09DC}');
                index += 2;
                continue;
            }
            ('\u{09A2}', Some(NUKTA)) => {
                out.push('\u{09DD}');
                index += 2;
                continue;
            }
            ('\u{09AF}', Some(NUKTA)) => {
                out.push('\u{09DF}');
                index += 2;
                continue;
            }
            (ZWNJ, _) => {}
            // র ZWJ ্ য is the one place the joiner carries meaning (the ra-phala-less "ryo" spelling).
            (ZWJ, Some(HASANTA))
                if out.ends_with('\u{09B0}') && composed.get(index + 2) == Some(&'\u{09AF}') =>
            {
                out.push(ZWJ);
            }
            (ZWJ, _) => {}
            _ => out.push(c),
        }
        index += 1;
    }
    out
}
