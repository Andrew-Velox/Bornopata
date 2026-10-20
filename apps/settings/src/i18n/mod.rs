//! The text of the program in English and Bangla. The English text stays at the call site and is the key: `t("Save")`
//! gives the Bangla text when Bangla is on and has an entry, and the English text otherwise.

mod bangla;

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

static BANGLA_ON: AtomicBool = AtomicBool::new(false);

static BANGLA: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| bangla::TABLE.iter().copied().collect());

/// Switches the text of the whole program; the next frame shows it.
pub fn use_bangla(on: bool) {
    BANGLA_ON.store(on, Ordering::Relaxed);
}

/// Whether the text is Bangla now.
pub fn is_bangla() -> bool {
    BANGLA_ON.load(Ordering::Relaxed)
}

/// `english`, or its Bangla text when Bangla is on.
pub fn t(english: &'static str) -> &'static str {
    if is_bangla() {
        translate(english)
    } else {
        english
    }
}

/// The Bangla text of `english`, or `english` itself when it has none.
fn translate(english: &'static str) -> &'static str {
    BANGLA.get(english).copied().unwrap_or(english)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_text_is_translated_and_unknown_text_stays_english() {
        assert_eq!(translate("Save"), "সংরক্ষণ করুন");
        assert_eq!(
            translate("A string with no entry"),
            "A string with no entry"
        );
    }

    #[test]
    fn no_english_text_is_listed_twice() {
        let mut seen = std::collections::HashSet::new();
        for (english, _) in bangla::TABLE {
            assert!(seen.insert(*english), "{english}");
        }
    }

    #[test]
    fn every_entry_is_used_by_some_source_file() {
        let sources = [
            "src/page_id.rs",
            "src/shortcut.rs",
            "src/system/action.rs",
            "src/settings/phrases.rs",
        ];
        let mut text = String::new();
        for file in sources {
            text.push_str(&std::fs::read_to_string(file).unwrap());
        }
        for directory in ["src/view", "src/view/pages", "src/view/parts"] {
            for entry in std::fs::read_dir(directory).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().is_some_and(|extension| extension == "rs") {
                    text.push_str(&std::fs::read_to_string(path).unwrap());
                }
            }
        }
        for (english, _) in bangla::TABLE {
            assert!(text.contains(english), "no source uses: {english}");
        }
    }
}
