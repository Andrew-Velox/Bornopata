//! Typing, editing the composition, highlighting and committing.

use super::{Engine, Typed};
use crate::candidate::CandidateSource;

/// Characters that end a sentence: the context for the next word starts over after them.
const SENTENCE_ENDS: [char; 3] = ['.', '?', '!'];

impl Engine {
    /// Types one character. Letters (and Avro marks after a letter) join the word; anything else commits the
    /// highlighted candidate, if any, followed by the character converted by the Avro rules (`.` gives ।).
    pub fn type_char(&mut self, ch: char) -> Typed {
        if self.composition.accepts(ch) {
            self.composition.push(ch);
            self.refresh_candidates();
            return Typed::Composing;
        }
        let mut text = self.commit().unwrap_or_default();
        let mut buffer = [0u8; 4];
        text.push_str(&self.transliterator.convert(ch.encode_utf8(&mut buffer)));
        if SENTENCE_ENDS.contains(&ch) {
            self.previous = None;
        }
        Typed::Commit(text)
    }

    /// Deletes the last typed letter. Returns false when the composition was empty and the shell should
    /// pass the key to the application.
    pub fn backspace(&mut self) -> bool {
        if !self.composition.pop() {
            return false;
        }
        self.refresh_candidates();
        true
    }

    /// Moves the highlight by `delta`, wrapping around.
    pub fn move_highlight(&mut self, delta: isize) {
        let count = self.candidates.len() as isize;
        if count > 0 {
            self.highlighted = (self.highlighted as isize + delta).rem_euclid(count) as usize;
        }
    }

    /// Highlights candidate `index`; ignored when out of range.
    pub fn highlight(&mut self, index: usize) {
        if index < self.candidates.len() {
            self.highlighted = index;
        }
    }

    /// Commits the highlighted candidate and clears the composition. Returns `None` when nothing is being typed.
    /// The choice is remembered by the learner in memory; writing to disk is [`Self::flush`].
    pub fn commit(&mut self) -> Option<String> {
        let candidate = self.candidates.get(self.highlighted)?;
        let text = candidate.text.clone();
        // A phrase is typed on purpose and can be private (an address, a signature), and a date is only true today;
        // the learner never sees either.
        if self.learning
            && !matches!(
                candidate.source,
                CandidateSource::Phrase | CandidateSource::Shortcut
            )
        {
            self.learner
                .record(self.composition.as_str(), &text, self.previous.as_deref());
        }
        self.previous = Some(text.clone());
        self.composition.clear();
        self.candidates.clear();
        self.highlighted = 0;
        Some(text)
    }

    /// The user took a next-word suggestion: remembers the transition and makes the word the new context.
    /// Returns the text to commit.
    pub fn accept_next_word(&mut self, word: &str) -> String {
        if self.learning {
            self.learner.record("", word, self.previous.as_deref());
        }
        self.previous = Some(word.to_owned());
        word.to_owned()
    }

    /// Drops the composition without committing (Escape).
    pub fn reset(&mut self) {
        self.composition.clear();
        self.candidates.clear();
        self.highlighted = 0;
    }

    /// Forgets the sentence context, for example after the cursor moved or focus changed.
    pub fn forget_context(&mut self) {
        self.previous = None;
    }
}
