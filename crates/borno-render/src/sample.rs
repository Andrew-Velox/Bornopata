//! Sample frames, shared by the theme preview and the snapshot tests.

use crate::{Frame, Mode, Preedit, Row, Source, Tone};

/// What typing `bangl` looks like with a dictionary: the typed spelling first, then completions.
pub fn bangl() -> Frame {
    Frame {
        preedit: Some(Preedit::plain("bangl", 5)),
        rows: vec![
            row(0, "বাংল", Source::Typed, &[]),
            row(1, "বাংলাদেশ", Source::Completion, &[]),
            row(2, "বাংলা", Source::Completion, &[]),
            row(3, "বাংলাদেশের", Source::Completion, &[]),
            row(4, "বাংলাদেশী", Source::Completion, &[]),
        ],
        highlighted: Some(0),
        hint: None,
        mode: Mode::default(),
    }
}

/// What typing `bhalobasha` shows: a word chosen before with its gloss, emoji with their names, other spellings and
/// completions. The picture of the themes on the website and in the docs.
pub fn showcase() -> Frame {
    Frame {
        preedit: Some(Preedit::plain("bhalobasha", 10)),
        rows: vec![
            row(
                0,
                "ভালোবাসা",
                Source::Learned,
                &[("v. ", Tone::Pos), ("love", Tone::Gloss)],
            ),
            row(1, "🫶", Source::Typed, &[("হৃদয়াকৃতির হাত", Tone::Gloss)]),
            row(2, "🧡", Source::Typed, &[("কমলা হার্ট", Tone::Gloss)]),
            row(3, "ভালবাসা", Source::Typed, &[]),
            row(4, "ভালবাশা", Source::Typed, &[]),
            row(5, "ভালোবাসার", Source::Completion, &[]),
            row(6, "ভালোবাসায়", Source::Completion, &[]),
        ],
        highlighted: Some(0),
        hint: None,
        mode: Mode::default(),
    }
}

/// A word with a learned choice, conjuncts and a spelling variant, to check shaping.
pub fn shaping() -> Frame {
    Frame {
        preedit: Some(Preedit::plain("bishshobiddaloy", 15)),
        rows: vec![
            row(0, "বিশ্ববিদ্যালয়", Source::Learned, &[]),
            row(1, "বিশ্বিদ্যালয়", Source::Typed, &[]),
            row(2, "ক্ষমা", Source::Typed, &[]),
            row(3, "শিক্ষা", Source::Typed, &[("direct", Tone::Faint)]),
            row(4, "কোথায়", Source::Typed, &[]),
            row(5, "দুঃখ", Source::Typed, &[]),
            row(6, "চাঁদ", Source::Typed, &[]),
        ],
        highlighted: Some(0),
        hint: None,
        mode: Mode::default(),
    }
}

/// After a space: next-word suggestions and the key hint.
pub fn next_words() -> Frame {
    Frame {
        preedit: None,
        rows: vec![
            row(0, "ভাত", Source::Next, &[]),
            row(1, "যাই", Source::Next, &[]),
            row(2, "ভালো", Source::Next, &[]),
        ],
        highlighted: Some(0),
        hint: Some("Tab".to_owned()),
        mode: Mode::default(),
    }
}

/// A row with a source and annotation segments; the index counts from 0.
pub fn row(index: usize, text: &str, source: Source, annotation: &[(&str, Tone)]) -> Row {
    Row {
        index: (index + 1).to_string(),
        text: text.to_owned(),
        annotation: annotation
            .iter()
            .map(|(s, tone)| ((*s).to_owned(), *tone))
            .collect(),
        source,
    }
}
