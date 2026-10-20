//! Emoji for the words they stand for. The table is `assets/emoji/emoji.tsv` (Unicode CLDR names and keywords, see the
//! README there), compiled into the program; [`EmojiSet`] is Core's `EmojiSource`.

mod set;

#[cfg(test)]
mod tests;

pub use set::EmojiSet;
