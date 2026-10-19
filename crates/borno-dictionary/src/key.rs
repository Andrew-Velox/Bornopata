//! The phonetic key: a spelling reduced to what people type the same way.
//!
//! `okkhor` turns typed Roman into one Bangla spelling. People type loosely though: they forget the case that
//! separates `t` from `T`, write `sh` for ষ or স, double a vowel, skip the o of কোথা. Both the typed text (after
//! `okkhor`) and every dictionary word are reduced to the same key, so those variants meet.

use crate::normalize::canonical;

/// The key of `text`, which may be any Bengali text, normalized or not.
///
/// Letters people confuse share a class, long and short vowels and independent and dependent vowels share a
/// class, the o sign is dropped (it is the inherent vowel written out), the hasanta, the y-phala, the joiners and
/// the nasalization mark are dropped, and a run of the same consonant is one consonant.
pub fn key(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_consonant: Option<char> = None;
    let mut after_hasanta = false;
    for c in canonical(text).chars() {
        // The hasanta only says "these consonants are joined"; people type joined and unjoined alike, so it is
        // dropped, and the y-phala after it (ব্যবহার, বিদ্যালয়) is dropped too: it is the doubled sound.
        if c == '\u{09CD}' {
            after_hasanta = true;
            continue;
        }
        let was_after_hasanta = std::mem::take(&mut after_hasanta);
        if was_after_hasanta && c == 'য' {
            continue;
        }
        let Some(class) = class(c) else {
            continue;
        };
        if is_consonant_class(class) {
            // শশ, ন্ন and the like become one letter.
            if last_consonant == Some(class) {
                continue;
            }
            last_consonant = Some(class);
        } else {
            last_consonant = None;
        }
        // The same vowel twice in a row (typed "aa" after an a-sign) is one vowel.
        if is_vowel_class(class) && out.chars().next_back() == Some(class) {
            continue;
        }
        out.push(class);
    }
    out
}

fn is_consonant_class(c: char) -> bool {
    ('\u{0995}'..='\u{09B9}').contains(&c) || matches!(c, '\u{09DF}')
}

fn is_vowel_class(c: char) -> bool {
    matches!(
        c,
        '\u{09BE}'
            | '\u{09BF}'
            | '\u{09C1}'
            | '\u{09C7}'
            | '\u{09C8}'
            | '\u{09CC}'
            | '\u{0985}'
            | '\u{09C3}'
    )
}

/// The class representative for `c`, or `None` when `c` leaves no trace in the key.
fn class(c: char) -> Option<char> {
    Some(match c {
        // sibilants
        'শ' | 'ষ' | 'স' => 'স',
        // nasals
        'ন' | 'ণ' => 'ন',
        'ঙ' | 'ং' => 'ং',
        // ja and ya sound alike
        'জ' | 'য' => 'জ',
        // case-blind t, d, r sounds
        'ত' | 'ট' | '\u{09CE}' => 'ত',
        'থ' | 'ঠ' => 'থ',
        'দ' | 'ড' => 'দ',
        'ধ' | 'ঢ' => 'ধ',
        'র' | '\u{09DC}' | '\u{09DD}' => 'র',
        // vowels: independent and dependent forms, long and short
        'আ' | '\u{09BE}' => '\u{09BE}',
        'ই' | 'ঈ' | '\u{09BF}' | '\u{09C0}' => '\u{09BF}',
        'উ' | 'ঊ' | '\u{09C1}' | '\u{09C2}' => '\u{09C1}',
        'ঋ' | '\u{09C3}' | '\u{09C4}' => '\u{09C3}',
        'এ' | '\u{09C7}' => '\u{09C7}',
        'ঐ' | '\u{09C8}' => '\u{09C8}',
        'ঔ' | '\u{09CC}' | '\u{09D7}' => '\u{09CC}',
        // the o sign is the inherent vowel written out; a leading ও is typed like অ
        '\u{09CB}' => return None,
        'ও' | 'অ' => '\u{0985}',
        // nasalization and the joiners vanish
        '\u{0981}' | '\u{200C}' | '\u{200D}' => return None,
        other => other,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn same(a: &str, b: &str) {
        assert_eq!(key(a), key(b), "{a} / {b}");
    }

    fn different(a: &str, b: &str) {
        assert_ne!(key(a), key(b), "{a} / {b}");
    }

    #[test]
    fn confusable_letters_meet() {
        same("শিক্ষা", "সিক্ষা");
        same("কন", "কণ");
        same("জন", "যন");
        same("কত", "কট");
        same("বা\u{09DC}ি", "বারি");
        same("বাংলা", "বাঙলা");
    }

    #[test]
    fn vowel_length_form_and_o_sign_do_not_matter() {
        same("আমি", "আমী");
        same("দুর", "দূর");
        same("কথা", "কোথা");
        same("কথাআ", "কথা");
        same("ইসলাম", "ঈসলাম");
        same("এক", "একে".trim_end_matches('ে'));
    }

    #[test]
    fn doubled_consonants_hasanta_and_y_phala_collapse() {
        same("উত্তর", "উতর");
        same("ধন্যবাদ", "ধনবাদ");
        // What okkhor makes of "bishshobiddaloy" meets the dictionary spelling.
        same("বিশ্ববিদ্যালয়", "বিশশবিদ্দালয়");
        same("বিদ্যালয়", "বিদ্দালয়");
    }

    #[test]
    fn real_differences_survive() {
        different("আমি", "অমি");
        different("কথা", "কাজ");
        different("পাতা", "ফাতা");
        different("চাল", "ছাল");
        different("কাল", "কুল");
        different("ক্ষমা", "খমা");
    }

    #[test]
    fn the_key_of_a_prefix_is_a_prefix_of_the_key() {
        let word = key("বাংলাদেশ");
        for prefix in ["বা", "বাং", "বাংল", "বাংলা", "বাংলাদে"]
        {
            assert!(word.starts_with(&key(prefix)), "{prefix}");
        }
    }

    #[test]
    fn unnormalized_input_gives_the_same_key() {
        same("বাড\u{09BC}ি", "বা\u{09DC}ি");
        same("ক\u{200C}থা", "কথা");
        same("কে\u{09BE}থা", "কোথা");
    }
}
