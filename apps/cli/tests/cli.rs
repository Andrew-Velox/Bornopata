//! The CLI end to end: the real binary, the real engine path.

use std::path::PathBuf;
use std::process::{Command, Output};

/// Runs the CLI without any dictionary file from the real user directory, so the result does not depend on the
/// machine; tests that want a dictionary pass `--dict`.
fn run(args: &[&str]) -> Output {
    let hermetic = !args.contains(&"--dict");
    Command::new(env!("CARGO_BIN_EXE_borno-cli"))
        .args(hermetic.then_some("--no-dict"))
        .args(args)
        .output()
        .expect("the CLI starts")
}

fn stdout(args: &[&str]) -> String {
    let output = run(args);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn scratch(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("borno-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    directory
}

#[test]
fn ami_shows_the_word_first_with_its_source() {
    let output = stdout(&["ami"]);
    assert!(output.contains("1. আমি  [direct]"), "{output}");
    assert!(output.contains("text: আমি"), "{output}");
}

#[test]
fn several_words_become_a_sentence() {
    let output = stdout(&["ami", "bangla", "kotha"]);
    assert!(output.contains("text: আমি বাংলা কথা"), "{output}");
}

#[test]
fn a_conjunct_and_letter_case_reach_the_output() {
    let output = stdout(&["shikkha", "ta", "Ta"]);
    assert!(output.contains("text: শিক্ষা তা টা"), "{output}");
}

#[test]
fn no_words_is_not_an_error() {
    let output = stdout(&[]);
    assert!(output.starts_with("text:"), "{output}");
}

#[test]
fn a_very_long_word_still_answers() {
    let word = "a".repeat(3000);
    let output = stdout(&[&word]);
    assert!(output.contains("[direct]"), "{output}");
}

#[test]
fn picking_past_the_candidates_fails_with_a_message() {
    let output = run(&["--pick", "99", "ami"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--pick 99"));
}

#[test]
fn a_picked_candidate_is_learned_across_runs() {
    let directory = scratch("learn");
    let user_dir = directory.to_str().unwrap();

    let first = stdout(&["--user-dir", user_dir, "--pick", "2", "kotha"]);
    assert!(first.contains("1. কথা  [direct]"), "{first}");
    let picked = first
        .lines()
        .find_map(|line| line.strip_prefix("text: "))
        .expect("the committed text is printed")
        .to_owned();
    assert_ne!(picked, "কথা");

    let later = stdout(&["--user-dir", user_dir, "kotha"]);
    assert!(
        later.contains(&format!("1. {picked}  [learned]")),
        "{later}"
    );
    assert!(later.contains(&format!("text: {picked}")), "{later}");
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn next_words_follow_what_was_typed_before() {
    let directory = scratch("next");
    let user_dir = directory.to_str().unwrap();
    stdout(&["--user-dir", user_dir, "ami", "bhat"]);
    let output = stdout(&["--user-dir", user_dir, "ami"]);
    assert!(output.contains("next: ভাত"), "{output}");
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_dictionary_completes_what_is_typed_and_ranks_by_frequency() {
    let directory = scratch("dictionary");
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("words.bnd");
    let mut builder = borno_dictionary::DictionaryBuilder::new();
    for (word, count) in [
        ("বাংলা", 400),
        ("বাংলাদেশ", 9_000),
        ("বাংলাদেশী", 100),
        ("আমি", 5_000),
    ] {
        builder.add(word, count);
    }
    builder
        .write(&path, &borno_format::Metadata::default())
        .unwrap();

    let output = stdout(&["--dict", path.to_str().unwrap(), "bangl"]);
    assert!(output.contains("1. বাংল  [direct]"), "{output}");
    assert!(output.contains("2. বাংলাদেশ  [completion]"), "{output}");
    assert!(output.contains("3. বাংলা  [completion]"), "{output}");
    // Space commits what was typed, never a completion.
    assert!(output.contains("text: বাংল"), "{output}");

    let output = stdout(&["--dict", path.to_str().unwrap(), "bangla"]);
    assert!(output.contains("1. বাংলা  [direct]"), "{output}");
    assert!(output.contains("2. বাংলাদেশ  [completion]"), "{output}");
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_missing_dictionary_file_is_an_error() {
    let output = run(&["--dict", "/nonexistent/words.bnd", "ami"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("dictionary"));
}
