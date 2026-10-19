//! Turning Wiktionary entries into one short gloss per word.
//!
//! A word is glossed from its own entry when it has a usable one. Otherwise, or when it is a pronoun form, it is glossed
//! through the lemma it is an inflected form of, but only when that is a sound match: case forms of nouns and
//! pronouns, verb forms and spelling variants, never the forms of adjectives or a lemma that is a different word.

mod items;
mod note;
mod pos;
mod pronoun;

use std::collections::HashMap;

use crate::kaikki::{Index, Line};
use crate::prefer::Prefer;
use items::{Kind, items};
use note::{is_form_note, is_low_register};

/// Most meanings in one gloss.
const MAX_MEANINGS: usize = 2;

/// A gloss longer than this (in characters) keeps only its first meaning.
const MAX_TEXT_CHARS: usize = 36;

/// Where a gloss came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// The word's own entry.
    Own,

    /// The entry of this lemma, the word being an inflected form of it.
    Lemma(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub pos: &'static str,

    pub text: String,

    pub route: Route,
}

/// The meanings of one entry, before they are put into a gloss.
struct Meaning {
    /// Wiktextract's name of the part of speech.
    pos_name: String,

    abbreviation: &'static str,

    items: Vec<String>,
}

pub struct Cleaner<'a> {
    index: &'a Index,
    prefer: &'a Prefer,

    /// Corpus counts, to pick the likelier lemma when a form belongs to several.
    counts: &'a HashMap<String, u64>,
}

impl<'a> Cleaner<'a> {
    pub fn new(index: &'a Index, prefer: &'a Prefer, counts: &'a HashMap<String, u64>) -> Self {
        Self {
            index,
            prefer,
            counts,
        }
    }

    pub fn resolve(&self, word: &str) -> Option<Resolved> {
        let prefer = self.prefer.pos_of(word);
        let own = self.own(self.index.lines(word), prefer);
        let through = self.through_lemma(word, prefer);
        // A preferred part of speech is met by whichever route has it, the word's own entry first.
        let own_matches = own
            .as_ref()
            .is_some_and(|own| prefer.is_none_or(|pos| own.pos_name == pos));
        match (own, through) {
            (Some(own), _) if own_matches && prefer.is_some() => Some(own_gloss(&own)),
            (_, Some((meaning, lemma, tags))) if prefer.is_some() => {
                Some(form_gloss(&meaning, &lemma, &tags))
            }
            (Some(_), Some((meaning, lemma, tags))) if meaning.pos_name == "pron" => {
                Some(form_gloss(&meaning, &lemma, &tags))
            }
            (Some(own), _) => Some(own_gloss(&own)),
            (None, Some((meaning, lemma, tags))) => Some(form_gloss(&meaning, &lemma, &tags)),
            (None, None) => None,
        }
    }

    /// The meanings of the first usable entry of `lines`. Entries of a named part of speech come first when `prefer`
    /// says so; proper-name entries come after the others.
    fn own(&self, lines: &[Line], prefer: Option<&str>) -> Option<Meaning> {
        let mut usable: Vec<&Line> = lines
            .iter()
            .filter(|line| pos::abbreviation(&line.pos).is_some())
            .collect();
        usable.sort_by_key(|line| (prefer != Some(line.pos.as_str()), line.pos == "name"));
        usable.into_iter().find_map(|line| {
            let abbreviation = pos::abbreviation(&line.pos)?;
            let kind = if line.pos == "verb" {
                Kind::Verb
            } else {
                Kind::Other
            };
            let items = entry_items(line, kind);
            (!items.is_empty()).then(|| Meaning {
                pos_name: line.pos.clone(),
                abbreviation,
                items,
            })
        })
    }

    /// The gloss of the lemma `word` is a sound form of, with the lemma and the tags of the form. With `prefer`, only a
    /// lemma of that part of speech counts.
    fn through_lemma(
        &self,
        word: &str,
        prefer: Option<&str>,
    ) -> Option<(Meaning, String, Vec<String>)> {
        // (lemma, tags, whether the word's own entry says it) — the entry's own statement is the better evidence, so it
        // outranks how frequent the lemma is.
        let mut references: Vec<(String, Vec<String>, bool)> = Vec::new();
        for line in self.index.lines(word) {
            for sense in &line.senses {
                if let Some(lemma) = sense.lemma().and_then(borno_dictionary::normalize) {
                    references.push((lemma, sense.tags.clone(), true));
                }
            }
        }
        for form in self.index.forms(word) {
            references.push((form.lemma.clone(), form.tags.clone(), false));
        }
        references
            .into_iter()
            .filter(|(lemma, ..)| lemma != word)
            .filter_map(|(lemma, tags, stated)| {
                let meaning = self.own(self.index.lines(&lemma), prefer)?;
                let fits = prefer.is_none_or(|pos| meaning.pos_name == pos);
                (fits && sound(&meaning, &tags)).then_some((meaning, lemma, tags, stated))
            })
            .max_by_key(|(_, lemma, _, stated)| {
                (*stated, self.counts.get(lemma).copied().unwrap_or(0))
            })
            .map(|(meaning, lemma, tags, _)| (meaning, lemma, tags))
    }
}

/// The meanings of the senses of `line`: grammar notes are skipped, and so are rare senses unless nothing else is left.
fn entry_items(line: &Line, kind: Kind) -> Vec<String> {
    let collect = |allow_rare: bool| -> Vec<String> {
        let mut found: Vec<String> = Vec::new();
        for sense in &line.senses {
            let Some(gloss) = sense.gloss() else { continue };
            let rare = is_low_register(&sense.tags);
            let note = sense.lemma().is_some() || is_form_note(gloss);
            if note || (rare && !allow_rare) {
                continue;
            }
            for item in items(gloss, kind) {
                if !found
                    .iter()
                    .any(|seen: &String| seen.eq_ignore_ascii_case(&item))
                {
                    found.push(item);
                }
            }
        }
        found
    };
    let common = collect(false);
    if common.is_empty() {
        collect(true)
    } else {
        common
    }
}

/// Whether the lemma's meaning also fits a form of it with these tags.
fn sound(meaning: &Meaning, tags: &[String]) -> bool {
    let alternative = tags
        .iter()
        .any(|tag| tag == "alternative" || tag == "alt-of");
    match meaning.pos_name.as_str() {
        "noun" | "name" | "verb" | "pron" => true,
        _ => alternative,
    }
}

fn own_gloss(own: &Meaning) -> Resolved {
    Resolved {
        pos: own.abbreviation,
        text: join(&own.items, MAX_MEANINGS),
        route: Route::Own,
    }
}

/// The gloss of an inflected form: the lemma's meaning, with "of" for the genitive and "in" for the locative; a
/// pronoun in the genitive or objective is put into the English case ("his, her").
fn form_gloss(meaning: &Meaning, lemma: &str, tags: &[String]) -> Resolved {
    let has = |wanted: &str| tags.iter().any(|tag| tag == wanted);
    let case = meaning.pos_name == "pron" && (has("genitive") || has("objective"));
    let pronouns: Option<Vec<String>> = case
        .then(|| {
            meaning
                .items
                .iter()
                .take(MAX_MEANINGS)
                .map(|item| pronoun::inflect(item, has("genitive")).map(str::to_owned))
                .collect()
        })
        .flatten();
    let text = if let Some(pronouns) = pronouns {
        pronouns.join(", ")
    } else if has("genitive") {
        format!("of {}", join(&meaning.items, 1))
    } else if has("locative") {
        format!("in {}", join(&meaning.items, 1))
    } else {
        join(&meaning.items, MAX_MEANINGS)
    };
    Resolved {
        pos: meaning.abbreviation,
        text,
        route: Route::Lemma(lemma.to_owned()),
    }
}

/// The first `count` meanings joined by commas; only the first one when the joined text would be long.
fn join(items: &[String], count: usize) -> String {
    let joined = items
        .iter()
        .take(count)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    if joined.chars().count() > MAX_TEXT_CHARS {
        items[0].clone()
    } else {
        joined
    }
}

#[cfg(test)]
mod tests;
