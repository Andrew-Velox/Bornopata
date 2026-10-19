//! Errors of the CLI.

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("failed to write output: {0}")]
    Output(#[from] std::io::Error),

    #[error("failed to open user data: {0}")]
    Learning(#[from] borno_learning::LearningError),

    #[error("failed to open the dictionary: {0}")]
    Dictionary(#[from] borno_dictionary::DictionaryError),

    #[error("failed to load language model: {0}")]
    LanguageModel(#[from] borno_lm::LmError),

    #[error("the language model needs both --lm-unigram and --lm-bigram")]
    IncompleteLanguageModel,

    #[error("--pick {pick} is out of range: the word has {count} candidates")]
    PickOutOfRange { pick: usize, count: usize },
}
