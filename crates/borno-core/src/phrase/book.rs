//! The phrases parsed from a file, and the file format.

use std::collections::HashMap;

use super::{PhraseProblem, PhraseSource};

/// Phrases by code. Codes are matched exactly as typed, case included, because Avro treats `t` and `T` as different
/// letters.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhraseBook {
    phrases: HashMap<String, Vec<String>>,
}

/// The phrases of a file and the lines that were skipped.
#[derive(Debug, Default)]
pub struct Parsed {
    pub book: PhraseBook,
    pub problems: Vec<PhraseProblem>,
}

impl PhraseBook {
    /// Reads `code<TAB>text` lines. Blank lines and lines starting with `#` are ignored. A code is Roman letters only,
    /// since nothing else can be typed as part of a word. In the text `\n`, `\t` and `\\` stand for a line break, a tab
    /// and a backslash. Several lines with one code give several candidates, in file order. A bad line is skipped and
    /// reported; the rest still loads.
    pub fn parse(text: &str) -> Parsed {
        let mut parsed = Parsed::default();
        for (index, line) in text.lines().enumerate() {
            let line = line.trim_end_matches('\r');
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let problem = |reason| PhraseProblem {
                line: index + 1,
                reason,
            };
            let Some((code, phrase)) = line.split_once('\t') else {
                parsed
                    .problems
                    .push(problem("expected a code, a tab and the text"));
                continue;
            };
            if code.is_empty() || !code.chars().all(|ch| ch.is_ascii_alphabetic()) {
                parsed
                    .problems
                    .push(problem("the code must be Roman letters only"));
                continue;
            }
            let phrase = unescape(phrase);
            if phrase.is_empty() {
                parsed.problems.push(problem("the text is empty"));
                continue;
            }
            let texts = parsed.book.phrases.entry(code.to_owned()).or_default();
            if !texts.contains(&phrase) {
                texts.push(phrase);
            }
        }
        parsed
    }

    pub fn len(&self) -> usize {
        self.phrases.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.phrases.is_empty()
    }
}

impl PhraseSource for PhraseBook {
    fn lookup(&self, code: &str) -> Vec<String> {
        self.phrases.get(code).cloned().unwrap_or_default()
    }
}

fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_codes_comments_and_repeated_codes() {
        let parsed = PhraseBook::parse(
            "# my phrases\n\nmymail\tme@example.com\nsig\tধন্যবাদ\nsig\tশুভেচ্ছা\nsig\tধন্যবাদ\n",
        );
        assert!(parsed.problems.is_empty());
        assert_eq!(parsed.book.len(), 3);
        assert_eq!(parsed.book.lookup("mymail"), ["me@example.com"]);
        assert_eq!(parsed.book.lookup("sig"), ["ধন্যবাদ", "শুভেচ্ছা"]);
    }

    #[test]
    fn codes_are_matched_exactly() {
        let parsed = PhraseBook::parse("Sig\tএক\n");
        assert_eq!(parsed.book.lookup("Sig"), ["এক"]);
        assert!(parsed.book.lookup("sig").is_empty());
    }

    #[test]
    fn escapes_give_line_breaks_tabs_and_backslashes() {
        let parsed = PhraseBook::parse("sign\tRegards,\\nMohabbat\\t\\\\ok\\q\n");
        assert_eq!(parsed.book.lookup("sign"), ["Regards,\nMohabbat\t\\ok\\q"]);
    }

    #[test]
    fn a_bad_line_is_reported_and_the_rest_loads() {
        let parsed = PhraseBook::parse("no tab here\nmy1\tx\n\ttext\nok\tfine\nempty\t\n");
        assert_eq!(parsed.book.lookup("ok"), ["fine"]);
        let lines: Vec<usize> = parsed.problems.iter().map(|p| p.line).collect();
        assert_eq!(lines, [1, 2, 3, 5]);
        assert_eq!(
            parsed.problems[0].to_string(),
            "line 1: expected a code, a tab and the text"
        );
    }

    #[test]
    fn windows_line_endings_are_fine() {
        let parsed = PhraseBook::parse("a\tone\r\nb\ttwo\r\n");
        assert_eq!(parsed.book.lookup("a"), ["one"]);
        assert_eq!(parsed.book.lookup("b"), ["two"]);
    }
}
