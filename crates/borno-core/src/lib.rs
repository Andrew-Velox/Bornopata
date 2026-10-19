//! Bornopata Core: composition, candidate generation, ranking, and the seams to learning and language models.
//!
//! Platform-independent. A shell feeds typed characters to [`Engine`] and draws the candidates it returns.
//! Test for the boundary: swapping Fcitx5 for another framework must not need a change here.

pub mod candidate;
pub mod composition;
pub mod engine;
pub mod learner;
pub mod lm;
pub mod storage;

pub use candidate::{Candidate, CandidateSource};
pub use composition::Composition;
pub use engine::{Engine, Query, Timings, Typed};
pub use learner::{Learner, NoLearner};
pub use lm::{LanguageModel, NoLanguageModel};
