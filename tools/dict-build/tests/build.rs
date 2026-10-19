//! The real binary on the committed samples: build, open, look up.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use borno_dictionary::{Dictionary, key};

fn samples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/dictionary")
}

fn scratch(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("borno-dict-build-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_borno-dict-build"))
        .args(args)
        .output()
        .expect("the tool starts")
}

#[test]
fn builds_a_dictionary_from_the_sample_counts_and_corpus() {
    let directory = scratch("sample");
    let output = directory.join("words.bnd");
    let result = run(&[
        "--output",
        output.to_str().unwrap(),
        "--counts",
        samples().join("sample-words.tsv").to_str().unwrap(),
        "--corpus",
        samples().join("sample-corpus.txt").to_str().unwrap(),
        "--min-count",
        "1",
        "--license",
        "CC0-1.0",
        "--name",
        "sample",
        "--stats",
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report = String::from_utf8_lossy(&result.stderr);
    assert!(report.contains("distinct words"), "{report}");

    let dictionary = Dictionary::open(&output).unwrap();
    assert_eq!(dictionary.metadata().license, "CC0-1.0");
    assert!(
        dictionary
            .metadata()
            .generator
            .starts_with("borno-dict-build")
    );
    let top = dictionary.lookup(&key("আমি"), 3);
    assert_eq!((top[0].text, top[0].exact), ("আমি", true));
    // A corpus word that is not in the counts list is present too.
    assert!(
        dictionary
            .lookup(&key("যাচ্ছি"), 1)
            .iter()
            .any(|m| m.text == "যাচ্ছি")
    );
    // Prefix completion comes from the same file.
    let completions = dictionary.lookup(&key("বাংল"), 4);
    assert_eq!(completions[0].text, "বাংলাদেশ");
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn min_count_and_max_words_trim_the_list() {
    let directory = scratch("trim");
    let output = directory.join("words.bnd");
    let result = run(&[
        "--output",
        output.to_str().unwrap(),
        "--counts",
        samples().join("sample-words.tsv").to_str().unwrap(),
        "--max-words",
        "10",
        "--min-count",
        "5000",
        "--license",
        "CC0-1.0",
    ]);
    assert!(result.status.success());
    let dictionary = Dictionary::open(&output).unwrap();
    assert!(dictionary.len() <= 10 && dictionary.len() > 3);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn bad_input_fails_with_a_message() {
    let directory = scratch("bad");
    let output = directory.join("words.bnd");
    let out = output.to_str().unwrap();

    let none = run(&["--output", out, "--license", "CC0-1.0"]);
    assert!(!none.status.success());
    assert!(String::from_utf8_lossy(&none.stderr).contains("no input"));

    let bad = directory.join("bad.tsv");
    std::fs::write(&bad, "আমি\tmany\n").unwrap();
    let result = run(&[
        "--output",
        out,
        "--counts",
        bad.to_str().unwrap(),
        "--license",
        "CC0-1.0",
    ]);
    assert!(String::from_utf8_lossy(&result.stderr).contains("bad.tsv:1"));

    let latin = directory.join("latin.txt");
    std::fs::write(&latin, "only latin text here\n").unwrap();
    let result = run(&[
        "--output",
        out,
        "--corpus",
        latin.to_str().unwrap(),
        "--license",
        "CC0-1.0",
    ]);
    assert!(String::from_utf8_lossy(&result.stderr).contains("no word is left"));

    let missing = run(&[
        "--output",
        out,
        "--corpus",
        "/nonexistent/corpus",
        "--license",
        "CC0-1.0",
    ]);
    assert!(String::from_utf8_lossy(&missing.stderr).contains("/nonexistent/corpus"));
    assert!(!output.exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn reads_the_corpus_from_standard_input() {
    use std::io::Write;
    use std::process::Stdio;
    let directory = scratch("stdin");
    let output = directory.join("words.bnd");
    let mut child = Command::new(env!("CARGO_BIN_EXE_borno-dict-build"))
        .args([
            "--output",
            output.to_str().unwrap(),
            "--corpus",
            "-",
            "--min-count",
            "1",
            "--license",
            "CC0-1.0",
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("আমি তুমি আমি\n".as_bytes())
        .unwrap();
    assert!(child.wait_with_output().unwrap().status.success());
    let dictionary = Dictionary::open(&output).unwrap();
    assert_eq!(dictionary.lookup(&key("আমি"), 1)[0].frequency, 2);
    std::fs::remove_dir_all(directory).unwrap();
}
