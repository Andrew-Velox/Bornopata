//! The shared emoji table as an engine's own `EmojiSource`.

use std::sync::Arc;

use borno_core::EmojiSource;
use borno_emoji::EmojiSet;

pub(super) struct SharedEmoji(pub Arc<EmojiSet>);

impl EmojiSource for SharedEmoji {
    fn lookup(&self, key: &str) -> Vec<String> {
        self.0.lookup(key)
    }

    fn name(&self, emoji: &str) -> Option<String> {
        self.0.name(emoji)
    }
}
