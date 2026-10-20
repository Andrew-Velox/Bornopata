//! The emoji table and its keyword index.

use std::collections::HashMap;

use borno_core::EmojiSource;
use borno_dictionary::normalize;

/// The table built by `tools/emoji-build`.
const BUNDLED: &str = include_str!("../../../assets/emoji/emoji.tsv");

/// Emoji kept per keyword; the engine offers only the first few.
const MAX_EMOJI_PER_KEYWORD: usize = 8;

struct Entry {
    emoji: String,
    name: String,

    /// How many keywords the emoji has. Few means a plain, central emoji (the red heart) and many a detailed one
    /// (a face with heart eyes), so for a shared keyword the plain one goes first.
    keywords: usize,
}

/// Emoji by Bangla and English keyword. Never changes after it is built, so engines share one behind an `Arc`.
pub struct EmojiSet {
    entries: Vec<Entry>,

    /// Keyword to the entries it names: those with that exact name first, then by fewest keywords, then Unicode's
    /// order.
    index: HashMap<String, Vec<u32>>,
}

impl EmojiSet {
    /// The table that ships with the program.
    pub fn bundled() -> Self {
        Self::from_tsv(BUNDLED)
    }

    /// Reads rows of `emoji<TAB>Bangla name<TAB>Bangla keywords<TAB>English name<TAB>English keywords`, keywords
    /// separated by `|`. Comment lines start with `#`; bad rows are skipped.
    pub fn from_tsv(text: &str) -> Self {
        let mut entries = Vec::new();
        let mut named: Vec<(String, u32)> = Vec::new();
        let mut keyworded: Vec<(String, u32)> = Vec::new();
        for line in text.lines() {
            if line.starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            let [emoji, bn_name, bn_keywords, en_name, en_keywords] = fields.as_slice() else {
                continue;
            };
            if emoji.is_empty() {
                continue;
            }
            let id = entries.len() as u32;
            let name = if bn_name.is_empty() { en_name } else { bn_name };
            let keywords = bn_keywords.split('|').count() + en_keywords.split('|').count();
            entries.push(Entry {
                emoji: (*emoji).to_owned(),
                name: (*name).to_owned(),
                keywords,
            });
            named.extend(key(bn_name).map(|key| (key, id)));
            named.extend(key(en_name).map(|key| (key, id)));
            for keyword in bn_keywords.split('|').chain(en_keywords.split('|')) {
                keyworded.extend(key(keyword).map(|key| (key, id)));
            }
        }
        keyworded.sort_by_key(|(_, id)| (entries[*id as usize].keywords, *id));
        let mut index: HashMap<String, Vec<u32>> = HashMap::new();
        for (key, id) in named.into_iter().chain(keyworded) {
            let ids = index.entry(key).or_default();
            if ids.len() < MAX_EMOJI_PER_KEYWORD && !ids.contains(&id) {
                ids.push(id);
            }
        }
        Self { entries, index }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// The index key of one keyword: a single Bangla word in its canonical spelling, or a single lower-case English word.
/// A phrase gives none, since only one word at a time is ever typed.
fn key(keyword: &str) -> Option<String> {
    let keyword = keyword.trim();
    if keyword.is_empty() || keyword.contains(char::is_whitespace) {
        return None;
    }
    if keyword.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return Some(keyword.to_ascii_lowercase());
    }
    normalize(keyword)
}

impl EmojiSource for EmojiSet {
    fn lookup(&self, key: &str) -> Vec<String> {
        let found = self.index.get(key).or_else(|| {
            let canonical = normalize(key)?;
            self.index.get(&canonical)
        });
        found
            .into_iter()
            .flatten()
            .filter_map(|id| self.entries.get(*id as usize))
            .map(|entry| entry.emoji.clone())
            .collect()
    }

    fn name(&self, emoji: &str) -> Option<String> {
        self.entries
            .iter()
            .find(|entry| entry.emoji == emoji)
            .map(|entry| entry.name.clone())
            .filter(|name| !name.is_empty())
    }
}
