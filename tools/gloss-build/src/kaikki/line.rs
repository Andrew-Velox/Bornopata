//! The fields of a Kaikki (Wiktextract) JSONL line that the cleaning uses; the rest is ignored.

use serde::Deserialize;

/// One Wiktionary entry of one part of speech.
#[derive(Debug, Clone, Deserialize)]
pub struct Line {
    pub word: String,

    /// Wiktextract's name of the part of speech: `noun`, `verb`, `name`, `pron`, `postp`, `character`…
    pub pos: String,

    #[serde(default)]
    pub senses: Vec<Sense>,

    #[serde(default)]
    pub forms: Vec<Form>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sense {
    #[serde(default)]
    pub glosses: Vec<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub form_of: Vec<Link>,

    #[serde(default)]
    pub alt_of: Vec<Link>,
}

impl Sense {
    /// The most specific gloss: Wiktextract lists the parent sense first and the sub-sense last.
    pub fn gloss(&self) -> Option<&str> {
        self.glosses.last().map(String::as_str)
    }

    /// The sense says this word is a form of another one, and which.
    pub fn lemma(&self) -> Option<&str> {
        self.form_of
            .iter()
            .chain(&self.alt_of)
            .map(|link| link.word.as_str())
            .next()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Form {
    pub form: String,

    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Link {
    pub word: String,
}
