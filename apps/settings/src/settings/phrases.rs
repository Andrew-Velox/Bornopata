//! The phrase file as a list the program can edit, keeping its comments and every line it does not understand.

use std::path::PathBuf;

use borno_platform::paths;

enum Line {
    /// A comment, a blank line or anything else: kept as it is.
    Other(String),

    /// `code<TAB>text`, the text in the file's escaped form.
    Phrase { code: String, text: String },
}

pub struct Phrases {
    path: PathBuf,
    lines: Vec<Line>,
}

/// One phrase as the list shows it.
pub struct Entry {
    pub code: String,

    /// The text with real line breaks and tabs.
    pub text: String,
}

impl Phrases {
    pub fn load() -> Self {
        Self::from_file(paths::phrases_path())
    }

    pub fn from_file(path: PathBuf) -> Self {
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let lines = text
            .lines()
            .map(|line| {
                let line = line.trim_end_matches('\r');
                match line.split_once('\t') {
                    Some((code, text))
                        if !line.starts_with('#')
                            && !code.is_empty()
                            && code.chars().all(|c| c.is_ascii_alphabetic())
                            && !text.is_empty() =>
                    {
                        Line::Phrase {
                            code: code.to_owned(),
                            text: text.to_owned(),
                        }
                    }
                    _ => Line::Other(line.to_owned()),
                }
            })
            .collect();
        Self { path, lines }
    }

    pub fn entries(&self) -> Vec<Entry> {
        self.lines
            .iter()
            .filter_map(|line| match line {
                Line::Phrase { code, text } => Some(Entry {
                    code: code.clone(),
                    text: unescape(text),
                }),
                Line::Other(_) => None,
            })
            .collect()
    }

    /// Adds a phrase (`nth` is `None`) or replaces the nth, then writes the file.
    pub fn set(&mut self, nth: Option<usize>, code: &str, text: &str) -> Result<(), String> {
        let code = code.trim();
        if code.is_empty() || !code.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err("The code is Roman letters only, for example mymail".to_owned());
        }
        if text.is_empty() {
            return Err("The text is empty".to_owned());
        }
        let line = Line::Phrase {
            code: code.to_owned(),
            text: escape(text),
        };
        match nth.and_then(|nth| self.position(nth)) {
            Some(at) => self.lines[at] = line,
            None => self.lines.push(line),
        }
        self.save()
    }

    pub fn remove(&mut self, nth: usize) -> Result<(), String> {
        if let Some(at) = self.position(nth) {
            self.lines.remove(at);
        }
        self.save()
    }

    /// Where in the file the nth phrase is.
    fn position(&self, nth: usize) -> Option<usize> {
        self.lines
            .iter()
            .enumerate()
            .filter(|(_, line)| matches!(line, Line::Phrase { .. }))
            .nth(nth)
            .map(|(at, _)| at)
    }

    fn save(&self) -> Result<(), String> {
        let mut out = String::new();
        for line in &self.lines {
            match line {
                Line::Other(text) => out.push_str(text),
                Line::Phrase { code, text } => {
                    out.push_str(code);
                    out.push('\t');
                    out.push_str(text);
                }
            }
            out.push('\n');
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let temporary = self.path.with_extension("tsv.tmp");
        std::fs::write(&temporary, out).map_err(|error| error.to_string())?;
        std::fs::rename(&temporary, &self.path).map_err(|error| error.to_string())
    }
}

fn unescape(text: &str) -> String {
    let mut out = String::new();
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

fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\r', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str, text: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("borno-settings-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("phrases.tsv");
        std::fs::write(&path, text).unwrap();
        path
    }

    #[test]
    fn comments_stay_and_phrases_are_listed() {
        let path = scratch("list", "# mine\n\nmymail\tme@example.com\nsig\ta\\nb\n");
        let phrases = Phrases::from_file(path);
        let entries = phrases.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].text, "a\nb");
    }

    #[test]
    fn adding_editing_and_removing_keep_the_rest_of_the_file() {
        let path = scratch("edit", "# mine\nmymail\tme@example.com\n");
        let mut phrases = Phrases::from_file(path.clone());
        phrases.set(None, "sig", "thanks,\nMe").unwrap();
        phrases.set(Some(0), "mymail", "you@example.com").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text, "# mine\nmymail\tyou@example.com\nsig\tthanks,\\nMe\n");
        phrases.remove(0).unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "# mine\nsig\tthanks,\\nMe\n"
        );
    }

    #[test]
    fn a_bad_code_or_empty_text_is_refused() {
        let path = scratch("bad", "");
        let mut phrases = Phrases::from_file(path.clone());
        assert!(phrases.set(None, "my mail", "x").is_err());
        assert!(phrases.set(None, "mail2", "x").is_err());
        assert!(phrases.set(None, "mail", "").is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "");
    }
}
