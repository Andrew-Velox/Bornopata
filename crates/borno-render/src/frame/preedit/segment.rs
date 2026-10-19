//! One piece of the preedit and how it is drawn.

use super::style::PreeditStyle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreeditSegment {
    pub text: String,
    pub style: PreeditStyle,
}
