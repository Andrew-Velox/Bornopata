//! The instructions given to the model, and reading its reply.

use serde::Deserialize;

use crate::clean::meanings;

/// Parts of speech the reply may use.
const POS: [&str; 13] = [
    "n.", "v.", "adj.", "adv.", "pron.", "postp.", "conj.", "num.", "det.", "int.", "part.",
    "clf.", "phr.",
];

pub const SYSTEM: &str = "You are a lexicographer writing a Bangla to English dictionary line for each word. The lines are \
shown in a Bangla input method, beside the candidate word, so they must be tiny.\n\
Rules:\n\
- \"pos\": the most common part of speech, exactly one of n. v. adj. adv. pron. postp. conj. num. det. int. part. clf. phr.\n\
- \"en\": one or two of the most common meanings, comma separated, at most 3 words each. Verbs in the base form without \
\"to\". No parentheses, no explanations, no examples.\n\
- A word that is an inflected form (for example a genitive like ভারতের, or a verb form like করেছে) gets the meaning of its base \
word, with \"of\" for the genitive (of India) and \"in\" for the locative.\n\
- Names of people, places and organisations: the usual English spelling.\n\
- If the word is not a real Bangla word (a typo, a fragment, markup) or you are not sure what it means, answer \
\"pos\": \"\" and \"en\": \"?\". Never guess.\n\
Answer with JSON only: {\"items\":[{\"w\":\"<the word exactly as given>\",\"pos\":\"n.\",\"en\":\"rice\"}]}, one item per \
input word, in the same order.";

#[derive(Debug, Deserialize)]
struct Reply {
    items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
    w: String,

    #[serde(default)]
    pos: String,

    #[serde(default)]
    en: String,
}

/// A gloss the model gave for a word.
#[derive(Debug, PartialEq, Eq)]
pub struct Answer {
    pub word: String,

    pub pos: String,

    pub text: String,
}

/// The user message for a batch: the words as a JSON array.
pub fn user_message(words: &[String]) -> String {
    serde_json::to_string(words).expect("strings serialize")
}

/// The glosses in `reply` for the words in `asked`. Words the model skipped, did not know (`?`), answered with another
/// word, or gave a part of speech or text that does not pass the same cleaning Wiktionary glosses get, are left out.
pub fn read_reply(reply: &str, asked: &[String]) -> Vec<Answer> {
    let (Some(start), Some(end)) = (reply.find('{'), reply.rfind('}')) else {
        return Vec::new();
    };
    let Ok(parsed) = serde_json::from_str::<Reply>(&reply[start..=end]) else {
        return Vec::new();
    };
    parsed
        .items
        .into_iter()
        .filter(|item| asked.contains(&item.w))
        .filter_map(|item| {
            let pos = item.pos.trim().to_owned();
            if item.en.trim() == "?" || !(pos.is_empty() || POS.contains(&pos.as_str())) {
                return None;
            }
            let text = meanings(&item.en, pos == "v.");
            (!text.is_empty()).then_some(Answer {
                word: item.w,
                pos,
                text,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asked() -> Vec<String> {
        vec!["ভাত".into(), "যাওয়া".into(), "ক্ক".into(), "পানি".into()]
    }

    #[test]
    fn good_items_are_kept_cleaned_and_unknown_ones_are_dropped() {
        let reply = r#"Here you go: {"items":[
            {"w":"ভাত","pos":"n.","en":"cooked rice (food)"},
            {"w":"যাওয়া","pos":"v.","en":"to go, to leave, to depart"},
            {"w":"ক্ক","pos":"","en":"?"},
            {"w":"অন্য","pos":"n.","en":"other"},
            {"w":"পানি","pos":"noun","en":"water"}]} done"#;
        let answers = read_reply(reply, &asked());
        assert_eq!(
            answers,
            [
                Answer {
                    word: "ভাত".into(),
                    pos: "n.".into(),
                    text: "cooked rice".into()
                },
                Answer {
                    word: "যাওয়া".into(),
                    pos: "v.".into(),
                    text: "go, leave".into()
                },
            ]
        );
    }

    #[test]
    fn a_reply_that_is_not_the_expected_json_gives_nothing() {
        assert!(read_reply("sorry, I cannot", &asked()).is_empty());
        assert!(read_reply("{\"items\": 3}", &asked()).is_empty());
        assert!(read_reply("", &asked()).is_empty());
    }

    #[test]
    fn the_batch_is_sent_as_a_json_array() {
        assert_eq!(user_message(&["ভাত".to_owned()]), "[\"ভাত\"]");
    }
}
