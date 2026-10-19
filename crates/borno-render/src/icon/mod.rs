//! Small vector icons a theme can use (`"icon": "star"`), each implementing the engine's [`bamboo_slip::Icon`].

mod star;

use std::sync::{Arc, LazyLock};

use serde::Deserialize;

use star::Star;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Icon {
    /// Marks a word chosen before.
    Star,
}

static STAR: LazyLock<Arc<dyn bamboo_slip::Icon>> = LazyLock::new(|| Arc::new(Star));

impl Icon {
    /// The painter handed to the scene tree.
    pub(crate) fn painter(self) -> Arc<dyn bamboo_slip::Icon> {
        match self {
            Self::Star => Arc::clone(&STAR),
        }
    }
}
