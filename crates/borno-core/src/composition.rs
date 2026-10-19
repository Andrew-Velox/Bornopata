//! The Roman letters typed for the current word.

/// Raw input of the word being composed. Case is kept: Avro treats `t` and `T` as different letters.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Composition {
    raw: String,
}

impl Composition {
    /// Whether `ch` belongs to the word. Letters always do; the Avro marks `` ` ``, `^` and `:` only
    /// after a letter, so a leading one is ordinary punctuation.
    pub fn accepts(&self, ch: char) -> bool {
        ch.is_ascii_alphabetic() || (!self.raw.is_empty() && matches!(ch, '`' | '^' | ':'))
    }

    pub fn push(&mut self, ch: char) {
        self.raw.push(ch);
    }

    /// Removes the last character; returns whether there was one.
    pub fn pop(&mut self) -> bool {
        self.raw.pop().is_some()
    }

    pub fn clear(&mut self) {
        self.raw.clear();
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_case_and_accepts_marks_only_after_a_letter() {
        let mut composition = Composition::default();
        assert!(!composition.accepts('^'));
        assert!(!composition.accepts('1'));
        for ch in "kOTha".chars() {
            assert!(composition.accepts(ch));
            composition.push(ch);
        }
        assert_eq!(composition.as_str(), "kOTha");
        assert!(composition.accepts('^'));
        assert!(!composition.accepts(' '));
    }

    #[test]
    fn pop_reports_whether_it_removed_anything() {
        let mut composition = Composition::default();
        assert!(!composition.pop());
        composition.push('a');
        assert!(composition.pop());
        assert!(composition.is_empty());
    }
}
