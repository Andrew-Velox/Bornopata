//! Learner behavior: counts, choices, the bigram, persistence and cooling.

use borno_core::Learner;

use super::FrequencyLearner;

fn temporary_directory(name: &str) -> std::path::PathBuf {
    let directory =
        std::env::temp_dir().join(format!("borno-learning-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    directory
}

#[test]
fn records_words_choices_and_pairs() {
    let mut learner = FrequencyLearner::in_memory();
    learner.record("ami", "আমি", None);
    learner.record("kotha", "কোথা", Some("আমি"));
    learner.record("kotha", "কোথা", Some("আমি"));
    learner.record("kotha", "কথা", Some("আমি"));

    assert_eq!(learner.weight("কোথা"), 2);
    assert_eq!(learner.choice_weight("kotha", "কোথা"), 2);
    assert_eq!(learner.choice_weight("ami", "কোথা"), 0);
    assert_eq!(
        learner.choices("kotha"),
        [("কোথা".to_owned(), 2), ("কথা".to_owned(), 1)]
    );
    assert_eq!(learner.pair_count("আমি", "কোথা"), 2);
    assert_eq!(learner.followers(Some("আমি"), 1), [("কোথা".to_owned(), 2)]);
    assert_eq!(learner.followers(None, 5), [("আমি".to_owned(), 1)]);
    assert!(learner.is_dirty());
}

#[test]
fn ignores_empty_text_and_empty_input_choices() {
    let mut learner = FrequencyLearner::in_memory();
    learner.record("x", "", None);
    assert_eq!(learner.word_count(), 0);
    learner.record("", "ক", None);
    assert_eq!(learner.weight("ক"), 1);
    assert!(learner.choices("").is_empty());
}

#[test]
fn saves_and_reloads_everything() {
    let directory = temporary_directory("round-trip");
    let mut learner = FrequencyLearner::open(&directory).unwrap();
    learner.record("kotha", "কোথা", Some("আমি"));
    learner.record("kotha", "কোথা", Some("আমি"));
    learner.flush().unwrap();
    assert!(!learner.is_dirty());

    let reloaded = FrequencyLearner::open(&directory).unwrap();
    assert_eq!(reloaded.weight("কোথা"), 2);
    assert_eq!(reloaded.choice_weight("kotha", "কোথা"), 2);
    assert_eq!(reloaded.pair_count("আমি", "কোথা"), 2);
    std::fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn a_fresh_directory_opens_empty_and_writes_nothing_until_flushed() {
    let directory = temporary_directory("fresh");
    let mut learner = FrequencyLearner::open(&directory).unwrap();
    assert_eq!(learner.word_count(), 0);
    learner.flush().unwrap();
    assert!(!directory.exists());
}

#[test]
fn malformed_lines_are_skipped_not_fatal() {
    let directory = temporary_directory("damaged");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("user-words.tsv"),
        "ভাত\t3\t20000\nbroken\nভাল\tx\t1\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("user-choices.tsv"),
        "bhat\tভাত\t3\t20000\nshort\n",
    )
    .unwrap();
    let mut learner = FrequencyLearner::open(&directory).unwrap();
    learner.set_half_life_days(0.0);
    assert_eq!(learner.word_count(), 1);
    assert_eq!(learner.choices("bhat").len(), 1);
    std::fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn old_choices_cool_down() {
    let directory = temporary_directory("cooling");
    std::fs::create_dir_all(&directory).unwrap();
    // Last used day 0 is decades ago: the effective weight has decayed to nothing, but the entry is still on file.
    std::fs::write(directory.join("user-choices.tsv"), "kotha\tকথা\t4\t0\n").unwrap();
    let mut learner = FrequencyLearner::open(&directory).unwrap();
    assert_eq!(learner.choice_weight("kotha", "কথা"), 0);
    assert!(learner.choices("kotha").is_empty());
    learner.set_half_life_days(0.0);
    assert_eq!(learner.choice_weight("kotha", "কথা"), 4);
    std::fs::remove_dir_all(&directory).unwrap();
}
