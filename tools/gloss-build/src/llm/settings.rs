//! The model endpoint and its key, from the environment or an ignored `.env` file; never from the command line.

use std::fmt;

/// Environment variables read: `BORNO_LLM_API_KEY`, `BORNO_LLM_BASE_URL` (an OpenAI-compatible endpoint, for example
/// `https://api.groq.com/openai/v1`) and `BORNO_LLM_MODEL`.
const KEY: &str = "BORNO_LLM_API_KEY";
const BASE_URL: &str = "BORNO_LLM_BASE_URL";
const MODEL: &str = "BORNO_LLM_MODEL";

#[derive(Clone)]
pub struct Settings {
    pub base_url: String,

    pub model: String,

    api_key: String,
}

/// What can be missing.
#[derive(Debug, PartialEq, Eq)]
pub enum Missing {
    /// No key: the caller skips the language model and leaves the words without a gloss.
    Key,

    Other(&'static str),
}

impl Settings {
    /// Settings from `lookup` (the process environment), falling back to the `KEY=VALUE` lines of `dotenv`.
    pub fn resolve(lookup: impl Fn(&str) -> Option<String>, dotenv: &str) -> Result<Self, Missing> {
        let file = parse_dotenv(dotenv);
        let get = |name: &str| {
            lookup(name)
                .filter(|value| !value.trim().is_empty())
                .or_else(|| {
                    file.iter()
                        .find(|(key, _)| key == name)
                        .map(|(_, value)| value.clone())
                })
                .filter(|value| !value.trim().is_empty())
        };
        let api_key = get(KEY).ok_or(Missing::Key)?;
        let base_url = get(BASE_URL).ok_or(Missing::Other(BASE_URL))?;
        let model = get(MODEL).ok_or(Missing::Other(MODEL))?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            model,
            api_key,
        })
    }

    /// The key, for the one place that sends it. It is never formatted or logged.
    pub(super) fn api_key(&self) -> &str {
        &self.api_key
    }
}

/// The key does not appear in debug output.
impl fmt::Debug for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Settings")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("api_key", &"<hidden>")
            .finish()
    }
}

/// `KEY=VALUE` lines; blank lines and `#` comments are skipped, `export ` is allowed, matching quotes are removed.
fn parse_dotenv(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            let line = line.strip_prefix("export ").unwrap_or(line);
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            let value = value.trim();
            let value = [('"', '"'), ('\'', '\'')]
                .iter()
                .find_map(|(open, close)| value.strip_prefix(*open)?.strip_suffix(*close))
                .unwrap_or(value);
            Some((key.trim().to_owned(), value.to_owned()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn settings_come_from_the_environment_first_then_the_dotenv_file() {
        let env = |name: &str| (name == KEY).then(|| "from-env".to_owned());
        let dotenv = "# comment\nBORNO_LLM_API_KEY=from-file\nexport BORNO_LLM_BASE_URL=\"https://example.test/v1/\"\nBORNO_LLM_MODEL='small-model'\n";
        let settings = Settings::resolve(env, dotenv).unwrap();
        assert_eq!(settings.api_key(), "from-env");
        assert_eq!(settings.base_url, "https://example.test/v1");
        assert_eq!(settings.model, "small-model");
    }

    #[test]
    fn a_missing_key_is_reported_apart_from_other_missing_settings() {
        assert_eq!(
            Settings::resolve(none, "BORNO_LLM_MODEL=m\nBORNO_LLM_BASE_URL=u").unwrap_err(),
            Missing::Key
        );
        assert_eq!(
            Settings::resolve(none, "BORNO_LLM_API_KEY=k\nBORNO_LLM_MODEL=m").unwrap_err(),
            Missing::Other(BASE_URL)
        );
        assert_eq!(
            Settings::resolve(
                none,
                "BORNO_LLM_API_KEY=  \nBORNO_LLM_BASE_URL=u\nBORNO_LLM_MODEL=m"
            )
            .unwrap_err(),
            Missing::Key
        );
    }

    #[test]
    fn the_key_never_shows_in_debug_output() {
        let settings = Settings::resolve(
            none,
            "BORNO_LLM_API_KEY=secret-value\nBORNO_LLM_BASE_URL=u\nBORNO_LLM_MODEL=m",
        )
        .unwrap();
        assert!(!format!("{settings:?}").contains("secret-value"));
    }
}
