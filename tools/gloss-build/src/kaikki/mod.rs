//! The Kaikki extract of English Wiktionary, indexed by canonical word and by inflected form.

mod line;

use std::collections::HashMap;
use std::path::Path;

use borno_dictionary::normalize;

pub use line::Line;

use crate::error::BuildError;

/// An inflected form listed in the table of a lemma.
#[derive(Debug, Clone)]
pub struct FormOf {
    pub lemma: String,

    pub tags: Vec<String>,
}

/// Wiktionary's Bengali entries, keyed by the canonical spelling of the word.
#[derive(Debug, Default)]
pub struct Index {
    lines: HashMap<String, Vec<Line>>,
    forms: HashMap<String, Vec<FormOf>>,
}

/// Tags of a forms-table row that are bookkeeping, not an inflected form.
const BOOKKEEPING: [&str; 4] = [
    "romanization",
    "table-tags",
    "inflection-template",
    "canonical",
];

impl Index {
    pub fn load(path: &Path) -> Result<Self, BuildError> {
        let text = std::fs::read_to_string(path).map_err(|source| BuildError::Read {
            path: path.to_owned(),
            source,
        })?;
        let mut index = Self::default();
        for (number, raw) in text.lines().enumerate() {
            if raw.trim().is_empty() {
                continue;
            }
            let line: Line = serde_json::from_str(raw).map_err(|error| BuildError::Line {
                path: path.to_owned(),
                line: number + 1,
                reason: error.to_string(),
            })?;
            index.add(line);
        }
        Ok(index)
    }

    pub fn add(&mut self, line: Line) {
        let Some(lemma) = normalize(&line.word) else {
            return;
        };
        for form in &line.forms {
            if form
                .tags
                .iter()
                .any(|tag| BOOKKEEPING.contains(&tag.as_str()))
            {
                continue;
            }
            if let Some(form_word) = normalize(&form.form).filter(|word| *word != lemma) {
                self.forms.entry(form_word).or_default().push(FormOf {
                    lemma: lemma.clone(),
                    tags: form.tags.clone(),
                });
            }
        }
        self.lines.entry(lemma).or_default().push(line);
    }

    /// The entries whose headword is `word`, in Wiktionary's order.
    pub fn lines(&self, word: &str) -> &[Line] {
        self.lines.get(word).map_or(&[], Vec::as_slice)
    }

    /// The lemmas that list `word` as one of their inflected forms.
    pub fn forms(&self, word: &str) -> &[FormOf] {
        self.forms.get(word).map_or(&[], Vec::as_slice)
    }
}
