//! An emoji source with no emoji.

use super::EmojiSource;

#[derive(Debug, Clone, Copy, Default)]
pub struct NoEmoji;

impl EmojiSource for NoEmoji {
    fn lookup(&self, _key: &str) -> Vec<String> {
        Vec::new()
    }
}
