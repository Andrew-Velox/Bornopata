//! One chat-completion request to an OpenAI-compatible endpoint, sent by `curl`.
//!
//! The key goes to `curl` as a header in a configuration read from its standard input, so it is in no argument list
//! (`ps` shows arguments) and in no file. It is never printed, and errors built here do not contain it.

use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use serde::Deserialize;

use super::settings::Settings;

/// Longest a single request may take.
const TIMEOUT_SECONDS: &str = "120";

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("could not run curl: {0}")]
    Curl(String),

    /// The endpoint refused the key or the request; waiting will not help.
    #[error("the endpoint answered HTTP {0}: check the key, the base URL and the model name")]
    Refused(u16),

    /// Rate limit or a server hiccup; worth trying again later.
    #[error("the endpoint answered HTTP {0}")]
    Busy(u16),

    #[error("the reply was not a chat completion")]
    Malformed,
}

pub struct Client {
    settings: Settings,

    /// How long to wait before each retry of a busy endpoint.
    waits: Vec<Duration>,
}

#[derive(Deserialize)]
struct Completion {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: Option<String>,
}

impl Client {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            waits: vec![
                Duration::from_secs(5),
                Duration::from_secs(20),
                Duration::from_secs(60),
            ],
        }
    }

    #[cfg(test)]
    pub fn without_waiting(mut self) -> Self {
        self.waits = vec![Duration::ZERO; 2];
        self
    }

    pub fn model(&self) -> &str {
        &self.settings.model
    }

    /// The model's reply to `system` and `user`, retrying a busy endpoint a few times.
    pub fn complete(&self, system: &str, user: &str) -> Result<String, LlmError> {
        let body = serde_json::json!({
            "model": self.settings.model,
            "temperature": 0.2,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
        })
        .to_string();
        let mut waits = self.waits.iter();
        loop {
            match self.once(&body) {
                Err(LlmError::Busy(_)) if waits.len() > 0 => {
                    std::thread::sleep(*waits.next().expect("checked above"));
                }
                other => return other,
            }
        }
    }

    fn once(&self, body: &str) -> Result<String, LlmError> {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let file = std::env::temp_dir().join(format!(
            "borno-gloss-request-{}-{}.json",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&file, body).map_err(|error| LlmError::Curl(error.to_string()))?;
        let result = self.run_curl(&file);
        let _ = std::fs::remove_file(&file);
        let (status, reply) = result?;
        match status {
            200 => {
                let completion: Completion =
                    serde_json::from_str(&reply).map_err(|_| LlmError::Malformed)?;
                completion
                    .choices
                    .into_iter()
                    .find_map(|choice| choice.message.content)
                    .ok_or(LlmError::Malformed)
            }
            429 | 500..=599 => Err(LlmError::Busy(status)),
            other => Err(LlmError::Refused(other)),
        }
    }

    /// Runs curl and returns the HTTP status and the body.
    fn run_curl(&self, body_file: &std::path::Path) -> Result<(u16, String), LlmError> {
        let mut child = Command::new("curl")
            .args([
                "--silent",
                "--show-error",
                "--max-time",
                TIMEOUT_SECONDS,
                "--config",
                "-",
            ])
            .args(["--write-out", "\n%{http_code}"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| LlmError::Curl(error.to_string()))?;
        let config = curl_config(&self.settings, body_file);
        child
            .stdin
            .take()
            .expect("stdin was piped")
            .write_all(config.as_bytes())
            .map_err(|error| LlmError::Curl(error.to_string()))?;
        let output = child
            .wait_with_output()
            .map_err(|error| LlmError::Curl(error.to_string()))?;
        if !output.status.success() {
            // curl's own message can name the URL but not the header it was given.
            return Err(LlmError::Curl(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ));
        }
        let text = String::from_utf8_lossy(&output.stdout).into_owned();
        let (reply, status) = text.rsplit_once('\n').ok_or(LlmError::Malformed)?;
        let status = status.trim().parse().map_err(|_| LlmError::Malformed)?;
        Ok((status, reply.to_owned()))
    }
}

/// The curl configuration, with the key in it. Values are quoted and escaped as curl's config format asks.
fn curl_config(settings: &Settings, body_file: &std::path::Path) -> String {
    let quote = |text: &str| format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""));
    format!(
        "url = {}\nrequest = \"POST\"\nheader = {}\nheader = {}\ndata-binary = {}\n",
        quote(&format!("{}/chat/completions", settings.base_url)),
        quote("Content-Type: application/json"),
        quote(&format!("Authorization: Bearer {}", settings.api_key())),
        quote(&format!("@{}", body_file.display())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(key: &str) -> Settings {
        Settings::resolve(
            |_| None,
            &format!("BORNO_LLM_API_KEY={key}\nBORNO_LLM_BASE_URL=http://127.0.0.1:1/v1\nBORNO_LLM_MODEL=m"),
        )
        .unwrap()
    }

    #[test]
    fn the_config_carries_the_key_and_escapes_what_curl_would_misread() {
        let config = curl_config(&settings("k\"ey"), std::path::Path::new("/tmp/body.json"));
        assert!(config.contains("url = \"http://127.0.0.1:1/v1/chat/completions\""));
        assert!(config.contains("header = \"Authorization: Bearer k\\\"ey\""));
        assert!(config.contains("data-binary = \"@/tmp/body.json\""));
    }

    #[test]
    fn a_dead_endpoint_is_an_error_that_does_not_contain_the_key() {
        let error = Client::new(settings("super-secret-key"))
            .without_waiting()
            .complete("s", "u")
            .unwrap_err();
        assert!(!error.to_string().contains("super-secret-key"), "{error}");
    }
}
