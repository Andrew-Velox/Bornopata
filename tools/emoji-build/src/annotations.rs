//! The names and keywords of one CLDR annotations file (`common/annotations/<language>.xml`).

use std::collections::HashMap;

/// What the file says about one emoji.
#[derive(Debug, Default, Clone)]
pub struct Annotation {
    /// The short name (`type="tts"`).
    pub name: String,

    pub keywords: Vec<String>,
}

/// Annotations by `cp`, which CLDR writes without U+FE0F.
pub struct Annotations(HashMap<String, Annotation>);

const VARIATION_SELECTOR: char = '\u{fe0f}';

impl Annotations {
    /// The files have one `<annotation cp="…">a | b</annotation>` per line, so no XML parser is needed.
    pub fn parse(xml: &str) -> Self {
        let mut found: HashMap<String, Annotation> = HashMap::new();
        for line in xml.lines() {
            let Some(rest) = line.trim().strip_prefix("<annotation cp=\"") else {
                continue;
            };
            let Some((cp, rest)) = rest.split_once('"') else {
                continue;
            };
            let (is_name, rest) = match rest.strip_prefix(" type=\"tts\">") {
                Some(rest) => (true, rest),
                None => match rest.strip_prefix('>') {
                    Some(rest) => (false, rest),
                    None => continue,
                },
            };
            let Some((text, _)) = rest.split_once("</annotation>") else {
                continue;
            };
            let entry = found.entry(unescape(cp)).or_default();
            if is_name {
                entry.name = unescape(text);
            } else {
                entry.keywords = text.split('|').map(|k| unescape(k.trim())).collect();
            }
        }
        Self(found)
    }

    /// The annotation of an emoji sequence, with or without U+FE0F.
    pub fn get(&self, sequence: &str) -> Option<&Annotation> {
        let key: String = sequence
            .chars()
            .filter(|ch| *ch != VARIATION_SELECTOR)
            .collect();
        self.0.get(&key)
    }
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_names_and_keywords_and_ignores_the_rest() {
        let xml = "<ldml>\n\t\t<annotation cp=\"❤\">লাল হার্ট | হৃদয়</annotation>\n\t\t<annotation cp=\"❤\" type=\"tts\">লাল হার্ট</annotation>\n\t\t<annotation cp=\"&amp;\">and | sign</annotation>\n</ldml>\n";
        let annotations = Annotations::parse(xml);
        let heart = annotations.get("❤\u{fe0f}").unwrap();
        assert_eq!(heart.name, "লাল হার্ট");
        assert_eq!(heart.keywords, ["লাল হার্ট", "হৃদয়"]);
        assert_eq!(annotations.get("&").unwrap().keywords, ["and", "sign"]);
        assert!(annotations.get("x").is_none());
    }
}
