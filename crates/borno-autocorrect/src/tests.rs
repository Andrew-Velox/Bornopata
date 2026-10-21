use borno_core::AutocorrectSource;

use crate::AutocorrectTable;

const SAMPLE: &str = "# header\n\
facebook\tফেসবুক\n\
tv\tটিভি\n\
data\tডেটা\n\
data\tডাটা\n\
data\tডেটা\n\
Bad\tx\n\
nobangla\t\n\
no tab\n";

fn table() -> AutocorrectTable {
    AutocorrectTable::from_tsv(SAMPLE)
}

#[test]
fn a_known_word_gets_its_spellings_in_order() {
    let table = table();
    assert_eq!(table.len(), 3, "bad rows are skipped");
    assert_eq!(table.lookup("facebook"), ["ফেসবুক"]);
    assert_eq!(table.lookup("data"), ["ডেটা", "ডাটা"]);
    assert!(table.lookup("facebok").is_empty());
}

#[test]
fn capitals_that_read_as_a_written_word_are_the_same_word() {
    let table = table();
    assert_eq!(table.lookup("Facebook"), ["ফেসবুক"]);
    assert_eq!(table.lookup("TV"), ["টিভি"]);
    assert_eq!(table.lookup("FACEBOOK"), ["ফেসবুক"]);
    assert!(
        table.lookup("FaceBook").is_empty(),
        "a capital inside the word is Avro spelling"
    );
    assert!(table.lookup("fACEBOOK").is_empty());
    assert!(table.lookup("face1book").is_empty());
}

#[test]
fn a_bangla_ending_follows_the_corrected_word() {
    let table = table();
    assert_eq!(table.lookup("facebooke"), ["ফেসবুকে"]);
    assert_eq!(table.lookup("facebooker"), ["ফেসবুকের"]);
    assert_eq!(table.lookup("facebookke"), ["ফেসবুককে"]);
    assert_eq!(table.lookup("facebookgulo"), ["ফেসবুকগুলো"]);
    assert_eq!(table.lookup("Facebookder"), ["ফেসবুকদের"]);
    assert_eq!(table.lookup("data"), ["ডেটা", "ডাটা"]);
    assert_eq!(table.lookup("datar"), ["ডেটার", "ডাটার"]);
}

#[test]
fn a_word_that_ends_in_a_vowel_takes_the_endings_that_fit() {
    let table = table();
    assert_eq!(table.lookup("tvte"), ["টিভিতে"]);
    assert_eq!(table.lookup("tvr"), ["টিভির"]);
    assert!(
        table.lookup("tve").is_empty(),
        "a vowel sign cannot follow ি, and the stem would be two letters"
    );
    assert!(
        table.lookup("datae").is_empty() || !table.lookup("datae").iter().any(|t| t.contains("াে"))
    );
}

#[test]
fn the_bundled_table_is_there_and_free_of_the_obvious_mistakes() {
    let table = AutocorrectTable::bundled();
    assert!(table.len() > 300);
    assert_eq!(table.lookup("computer"), ["কম্পিউটার"]);
    assert_eq!(table.lookup("computerke"), ["কম্পিউটারকে"]);
    let text = include_str!("../../../assets/autocorrect/autocorrect.tsv");
    for line in text.lines().filter(|line| !line.starts_with('#')) {
        let (word, bangla) = line.split_once('\t').expect("every row has a tab");
        assert!(word.bytes().all(|b| b.is_ascii_lowercase()), "{line}");
        assert!(
            !bangla.trim().is_empty() && bangla == bangla.trim(),
            "{line}"
        );
        assert!(
            bangla
                .chars()
                .all(|ch| ('\u{0980}'..='\u{09FF}').contains(&ch)),
            "{line}: only Bengali letters"
        );
    }
}
