use borno_dictionary::DictionaryError;
use borno_learning::LearningError;
use borno_lm::LmError;
use borno_neural::NeuralError;
use borno_platform::ConfigError;
use borno_predict::PredictError;
use borno_translate::GlossaryError;
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Cold(#[from] crate::cold::error::ColdError),

    #[error(transparent)]
    Dictionary(#[from] DictionaryError),

    #[error(transparent)]
    Neural(#[from] NeuralError),

    #[error(transparent)]
    Glossary(#[from] GlossaryError),

    #[error(transparent)]
    Learning(#[from] LearningError),

    /// 学习语言不是 en / ja / es / de。
    #[error("learning language must be en, ja, es or de, got {0:?}")]
    Language(String),

    /// 删用户词后读回核实：要删的词仍在 `user-words.tsv` 里，说明没写进磁盘。
    #[error("用户词删除未落盘：{0} 仍在 user-words.tsv 里")]
    UserWordsRemain(String),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Predict(#[from] PredictError),

    #[error(transparent)]
    LanguageModel(#[from] LmError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Replay(#[from] crate::replay::ReplayError),

    #[error(transparent)]
    Eval(#[from] crate::eval::EvalError),

    #[error(transparent)]
    Tune(#[from] crate::tuning::TuneError),
}
