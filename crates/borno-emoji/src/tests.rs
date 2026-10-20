use borno_core::EmojiSource;

use crate::EmojiSet;

const SAMPLE: &str = "# header\n\
😀\tমুখে দেঁতো হাসি\tদেঁতো হাসি|মুখ|হাসি\tgrinning face\tface|grin|smile\n\
😃\tবড় হাসি\tমুখ|হাসি\tgrinning face with big eyes\tface|smile\n\
❤️\tলাল হার্ট\tলাল হার্ট|হৃদয়\tred heart\theart|love|red\n\
broken row\n";

#[test]
fn bangla_and_english_keywords_find_their_emoji() {
    let set = EmojiSet::from_tsv(SAMPLE);
    assert_eq!(set.len(), 3);
    assert_eq!(set.lookup("হাসি"), ["😃", "😀"]);
    assert_eq!(set.lookup("হৃদয়"), ["❤️"]);
    assert_eq!(set.lookup("love"), ["❤️"]);
    assert_eq!(set.lookup("smile"), ["😃", "😀"]);
}

#[test]
fn phrases_and_unknown_words_find_nothing() {
    let set = EmojiSet::from_tsv(SAMPLE);
    assert!(set.lookup("লাল হার্ট").is_empty());
    assert!(set.lookup("ami").is_empty());
    assert!(set.lookup("").is_empty());
}

#[test]
fn a_shared_keyword_offers_the_plain_emoji_first_and_only_a_few() {
    let mut text = String::new();
    for index in 0..12 {
        let more = "|x".repeat(index);
        text.push_str(&format!(
            "{}\tনাম\tমুখ{more}\tname\tface\n",
            char::from_u32(0x1f600 + 11 - index as u32).unwrap()
        ));
    }
    let set = EmojiSet::from_tsv(&text);
    let found = set.lookup("মুখ");
    assert_eq!(found.len(), 8);
    assert_eq!(found[0], char::from_u32(0x1f600 + 11).unwrap().to_string());
}

#[test]
fn an_exact_name_comes_before_a_keyword_match() {
    let set = EmojiSet::from_tsv("😀\tহাসি মুখ\tহাসি\tgrin\tsmile\n😃\tহাসি\tগোল\tbig grin\tbig\n");
    assert_eq!(set.lookup("হাসি"), ["😃", "😀"]);
}

#[test]
fn the_name_is_bangla_with_english_as_the_fallback() {
    let set = EmojiSet::from_tsv("😀\tহাসি\tx\tgrin\ty\n😃\t\tx\tbig grin\ty\n");
    assert_eq!(set.name("😀").as_deref(), Some("হাসি"));
    assert_eq!(set.name("😃").as_deref(), Some("big grin"));
    assert_eq!(set.name("x"), None);
}

#[test]
fn the_bundled_table_answers_common_words() {
    let set = EmojiSet::bundled();
    assert!(set.len() > 1000);
    assert!(set.lookup("হৃদয়").contains(&"❤️".to_owned()));
    assert!(set.lookup("heart").iter().any(|emoji| emoji.contains('❤')));
}
