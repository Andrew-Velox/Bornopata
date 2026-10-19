//! Classes of Bengali-block characters, as far as spelling validity needs them.

pub const ZWNJ: char = '\u{200C}';
pub const ZWJ: char = '\u{200D}';
pub const HASANTA: char = '\u{09CD}';
pub const NUKTA: char = '\u{09BC}';

/// Consonant letters, including the precomposed ড় ঢ় য় and the khanda ta ৎ.
pub fn is_consonant(c: char) -> bool {
    matches!(
        c,
        '\u{0995}'..='\u{09B9}' | '\u{09CE}' | '\u{09DC}' | '\u{09DD}' | '\u{09DF}'
    )
}

/// Independent vowel letters.
pub fn is_vowel_letter(c: char) -> bool {
    matches!(
        c,
        '\u{0985}'
            ..='\u{098C}'
                | '\u{098F}'
                | '\u{0990}'
                | '\u{0993}'
                | '\u{0994}'
                | '\u{09E0}'
                | '\u{09E1}'
    )
}

/// Dependent vowel signs (kar), including the au length mark.
pub fn is_vowel_sign(c: char) -> bool {
    matches!(
        c,
        '\u{09BE}'..='\u{09C4}' | '\u{09C7}' | '\u{09C8}' | '\u{09CB}' | '\u{09CC}' | '\u{09D7}'
    )
}

/// Candrabindu, anusvara and visarga.
pub fn is_modifier(c: char) -> bool {
    matches!(c, '\u{0981}'..='\u{0983}')
}

/// Whether `c` can be part of a token at all: the Bengali block minus digits, plus the zero-width joiners that
/// the normalizer sorts out afterwards.
pub fn is_word_char(c: char) -> bool {
    (('\u{0980}'..='\u{09FF}').contains(&c) && !('\u{09E6}'..='\u{09EF}').contains(&c))
        || c == ZWNJ
        || c == ZWJ
}
