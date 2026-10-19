//! Bangla or English typing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Mode {
    /// Letters are converted to Bangla.
    #[default]
    Bangla,

    /// The input method passes every key through.
    English,
}

impl Mode {
    pub fn toggled(self) -> Self {
        match self {
            Self::Bangla => Self::English,
            Self::English => Self::Bangla,
        }
    }
}
