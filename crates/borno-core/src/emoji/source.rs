//! The seam between the engine and the emoji table.

/// Implemented by `borno-emoji`. Core only asks.
pub trait EmojiSource: Send {
    /// Emoji for a keyword, best first: a Bangla word, or an English word typed in Roman letters. Empty when there are
    /// none.
    fn lookup(&self, key: &str) -> Vec<String>;

    /// The name of an emoji, shown beside it in the candidate list.
    fn name(&self, _emoji: &str) -> Option<String> {
        None
    }
}
