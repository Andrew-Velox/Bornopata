//! The Bangla endings that may follow a corrected word: `facebook` + `e` is ফেসবুকে.

/// Roman ending and its Bangla spelling, longest first so `der` is tried before `r`.
const SUFFIXES: [(&str, &str); 13] = [
    ("gulo", "গুলো"),
    ("guli", "গুলি"),
    ("der", "দের"),
    ("ke", "কে"),
    ("te", "তে"),
    ("ra", "রা"),
    ("er", "ের"),
    ("ta", "টা"),
    ("ti", "টি"),
    ("e", "ে"),
    ("r", "র"),
    ("o", "ও"),
    ("i", "ই"),
];

/// Letters a word must have before an ending is looked for, so `be` is not `b` + `e`. Two letters are enough when
/// none is a vowel (`tv`), since only an abbreviation looks like that.
const MIN_STEM: usize = 3;

/// Every way `input` is a table word plus one ending: the word's Roman letters and the Bangla ending.
pub(super) fn split(input: &str) -> impl Iterator<Item = (&str, &'static str)> {
    SUFFIXES.iter().filter_map(move |(roman, bangla)| {
        let stem = input.strip_suffix(roman)?;
        let long_enough = stem.len() >= MIN_STEM
            || (stem.len() == 2 && !stem.bytes().any(|b| b"aeiou".contains(&b)));
        long_enough.then_some((stem, *bangla))
    })
}

/// Whether `ending` can follow `word`. A vowel sign cannot be attached to a word that already ends in a vowel (the
/// `ি` of টিভি), where Bangla uses `তে` or `র` instead.
pub(super) fn fits(word: &str, ending: &str) -> bool {
    let sign_ending = ending.starts_with('ে');
    let vowel_end = word
        .chars()
        .next_back()
        .is_some_and(|ch| matches!(ch, 'া' | 'ি' | 'ী' | 'ু' | 'ূ' | 'ে' | 'ৈ' | 'ো' | 'ৌ'));
    !(sign_ending && vowel_end)
}
