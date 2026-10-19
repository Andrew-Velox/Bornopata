//! Errors while starting the Server.

use borno_learning::LearningError;
use borno_lm::LmError;
use borno_platform::ConfigError;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("config: {0}")]
    Config(#[from] ConfigError),

    #[error("learned data: {0}")]
    Learning(#[from] LearningError),

    #[error("language model: {0}")]
    LanguageModel(#[from] LmError),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
