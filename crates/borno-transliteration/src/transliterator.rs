//! Direct Roman to Bangla conversion by the Avro phonetic rules.

use okkhor::parser::Parser;

/// Deterministic Avro conversion: one spelling per input, case-sensitive for `o i u d g j n r s t y z`
/// (`t`/`T`, `d`/`D`, `n`/`N`, `o`/`O` are different letters) and case-insensitive for the rest.
pub struct Transliterator {
    parser: Parser,
}

impl Transliterator {
    pub fn new() -> Self {
        Self {
            parser: Parser::new_phonetic(),
        }
    }

    /// Converts `roman`. Digits become Bangla digits; punctuation and spaces pass through.
    pub fn convert(&self, roman: &str) -> String {
        self.parser.convert(roman)
    }
}

impl Default for Transliterator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn convert(roman: &str) -> String {
        Transliterator::new().convert(roman)
    }

    #[test]
    fn converts_common_words() {
        assert_eq!(convert("ami"), "আমি");
        assert_eq!(convert("bangla"), "বাংলা");
        assert_eq!(convert("kotha"), "কথা");
    }

    #[test]
    fn builds_conjuncts() {
        assert_eq!(convert("shikkha"), "শিক্ষা");
        assert_eq!(convert("kShoma"), "ক্ষমা");
    }

    #[test]
    fn keeps_the_case_that_selects_a_letter() {
        assert_eq!(convert("ta"), "তা");
        assert_eq!(convert("Ta"), "টা");
        assert_eq!(convert("da"), "দা");
        assert_eq!(convert("Da"), "ডা");
        assert_eq!(convert("rn"), "রন");
        assert_eq!(convert("rN"), "রণ");
    }

    #[test]
    fn ignores_case_for_other_letters() {
        assert_eq!(convert("KA"), convert("ka"));
        assert_eq!(convert("BHA"), convert("bha"));
        assert_eq!(convert("Bangla"), convert("bangla"));
    }

    #[test]
    fn handles_empty_digits_and_punctuation() {
        assert_eq!(convert(""), "");
        assert_eq!(convert("123"), "১২৩");
        assert_eq!(convert("ami."), "আমি।");
    }

    #[test]
    fn converts_long_input() {
        let roman = "ami ".repeat(5000);
        let bangla = convert(&roman);
        assert_eq!(bangla, "আমি ".repeat(5000));
    }
}
