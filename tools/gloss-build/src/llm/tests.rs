//! The generate loop against a local HTTP server standing in for the model endpoint.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use super::{Client, Options, Settings, generate};
use crate::error::BuildError;
use crate::table;

/// What the server saw: the `Authorization` header and the JSON body of each request.
type Seen = Vec<(String, serde_json::Value)>;

/// Serves `statuses.len()` requests, answering each with its status; a 200 carries a chat completion whose gloss of the
/// n-th word is `gloss<letter n>`, except that `ক্ক` is answered with `?`.
fn server(statuses: Vec<u16>) -> (u16, JoinHandle<Seen>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = std::thread::spawn(move || {
        let mut seen = Seen::new();
        for status in statuses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0u8; 4096];
            let (head_end, length) = loop {
                let count = stream.read(&mut buffer).unwrap();
                request.extend_from_slice(&buffer[..count]);
                if let Some(at) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&request[..at]).to_lowercase();
                    let length = head
                        .lines()
                        .find_map(|line| line.strip_prefix("content-length:"))
                        .map_or(0, |value| value.trim().parse::<usize>().unwrap());
                    break (at + 4, length);
                }
            };
            while request.len() < head_end + length {
                let count = stream.read(&mut buffer).unwrap();
                request.extend_from_slice(&buffer[..count]);
            }
            let head = String::from_utf8_lossy(&request[..head_end]).into_owned();
            let authorization = head
                .lines()
                .find_map(|line| line.strip_prefix("Authorization: "))
                .unwrap_or("")
                .to_owned();
            let body: serde_json::Value =
                serde_json::from_slice(&request[head_end..head_end + length]).unwrap();
            let words: Vec<String> =
                serde_json::from_str(body["messages"][1]["content"].as_str().unwrap()).unwrap();
            let items: Vec<serde_json::Value> = words
                .iter()
                .enumerate()
                .map(|(n, word)| {
                    let en = if word == "ক্ক" {
                        "?".to_owned()
                    } else {
                        format!("gloss{}", char::from(b'a' + n as u8))
                    };
                    serde_json::json!({"w": word, "pos": "n.", "en": en})
                })
                .collect();
            let content = serde_json::json!({"items": items}).to_string();
            let reply = if status == 200 {
                serde_json::json!({"choices": [{"message": {"content": content}}]}).to_string()
            } else {
                "{\"error\":\"no\"}".to_owned()
            };
            write!(
                stream,
                "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}",
                reply.len()
            )
            .unwrap();
            seen.push((authorization, body));
        }
        seen
    });
    (port, handle)
}

fn client(port: u16) -> Client {
    let dotenv = format!(
        "BORNO_LLM_API_KEY=test-key\nBORNO_LLM_BASE_URL=http://127.0.0.1:{port}/v1\nBORNO_LLM_MODEL=test-model"
    );
    Client::new(Settings::resolve(|_| None, &dotenv).unwrap()).without_waiting()
}

fn scratch(name: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "borno-gloss-llm-{}-{}-{name}.tsv",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn options(output: PathBuf, existing: Vec<PathBuf>, batch: usize) -> Options {
    let counts: HashMap<String, u64> =
        [("ভাত", 50), ("পানি", 40), ("ক্ক", 30), ("মাছ", 20), ("দুধ", 10)]
            .iter()
            .map(|(word, count)| ((*word).to_owned(), *count))
            .collect();
    Options {
        counts,
        top: 5,
        existing,
        output,
        batch,
        delay: Duration::ZERO,
        max_requests: None,
    }
}

#[test]
fn missing_words_are_asked_in_batches_and_the_answers_are_appended_with_the_model_as_source() {
    let known = scratch("known");
    std::fs::write(&known, "ভাত\tn.\trice\twiktionary\n").unwrap();
    let output = scratch("out");
    let (port, seen) = server(vec![200, 200]);
    let report = generate(
        &client(port),
        &options(output.clone(), vec![known.clone()], 2),
    )
    .unwrap();
    // ভাত has a gloss; পানি, ক্ক, মাছ, দুধ are asked in two batches of two; ক্ক is answered "?".
    assert_eq!((report.requests, report.asked, report.glossed), (2, 4, 3));
    let entries = table::read(&output).unwrap();
    let words: Vec<&str> = entries.iter().map(|e| e.word.as_str()).collect();
    assert_eq!(words, ["পানি", "মাছ", "দুধ"]);
    assert!(
        entries
            .iter()
            .all(|e| e.source == "llm:test-model" && e.pos == "n.")
    );
    let seen = seen.join().unwrap();
    assert!(
        seen.iter()
            .all(|(authorization, _)| authorization == "Bearer test-key")
    );
    assert!(
        seen.iter().all(
            |(_, body)| !body.to_string().contains("test-key") && body["model"] == "test-model"
        )
    );

    // A second run has nothing left to ask except the word the model did not know, which is asked again.
    let (port, seen) = server(vec![200]);
    let again = generate(
        &client(port),
        &options(output.clone(), vec![known.clone()], 2),
    )
    .unwrap();
    assert_eq!((again.requests, again.asked), (1, 1));
    seen.join().unwrap();
    std::fs::remove_file(known).unwrap();
    std::fs::remove_file(output).unwrap();
}

#[test]
fn a_refused_key_stops_with_an_error_that_does_not_contain_the_key() {
    let output = scratch("refused");
    let (port, seen) = server(vec![401]);
    let error = generate(&client(port), &options(output.clone(), Vec::new(), 5)).unwrap_err();
    let BuildError::Llm(message) = &error else {
        panic!("expected an Llm error, got {error}");
    };
    assert!(message.contains("401") && !message.contains("test-key"));
    seen.join().unwrap();
    assert!(table::read(&output).unwrap().is_empty());
    std::fs::remove_file(output).unwrap();
}

#[test]
fn a_busy_endpoint_is_retried_and_then_given_up_without_losing_what_was_written() {
    let output = scratch("busy");
    // The first batch works; the second gets 503 on the request and on both retries.
    let (port, seen) = server(vec![200, 503, 503, 503]);
    let report = generate(&client(port), &options(output.clone(), Vec::new(), 2)).unwrap();
    assert_eq!(report.glossed, 2, "{:?}", table::read(&output));
    assert_eq!(table::read(&output).unwrap().len(), 2);
    seen.join().unwrap();
    std::fs::remove_file(output).unwrap();
}

#[test]
fn a_run_can_be_limited_to_a_number_of_requests() {
    let output = scratch("limited");
    let (port, seen) = server(vec![200]);
    let mut limited = options(output.clone(), Vec::new(), 2);
    limited.max_requests = Some(1);
    let report = generate(&client(port), &limited).unwrap();
    assert_eq!(report.requests, 1);
    seen.join().unwrap();
    std::fs::remove_file(output).unwrap();
}
