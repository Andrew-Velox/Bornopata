//! A light cleaner for wiki markup, so that template and parameter names do not count as words.
//!
//! A Wikipedia dump is full of `|তারিখ=` and `{{উদ্ধৃতি ওয়েব` style names that are real Bengali words but occur millions
//! of times for reasons that have nothing to do with how people write. The cleaner works on one line at a time:
//! it needs no parser, and it errs on the side of dropping a little real text.

/// Link prefixes whose whole link is dropped: categories and files, in Bengali and English.
const DROPPED_LINKS: [&str; 8] = [
    "বিষয়শ্রেণী:",
    "ফাইল:",
    "চিত্র:",
    "চিত্র :",
    "category:",
    "file:",
    "image:",
    "বিষয়শ্রেণী :",
];

/// The longest parameter or template name removed, in characters.
const MAX_NAME_CHARS: usize = 40;

/// `line` without tags, `{{template` names, `|parameter=` names and category or file links.
pub fn strip(line: &str) -> String {
    let without_tags = strip_tags(line);
    let without_links = strip_links(&without_tags);
    strip_names(&without_links)
}

fn strip_tags(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_tag = false;
    for c in line.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if in_tag => {}
            _ => out.push(c),
        }
    }
    out
}

fn strip_links(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(start) = rest.find("[[") {
        out.push_str(&rest[..start]);
        let inside = &rest[start + 2..];
        let lower = inside.trim_start().to_lowercase();
        if DROPPED_LINKS.iter().any(|prefix| lower.starts_with(prefix)) {
            match inside.find("]]") {
                Some(end) => rest = &inside[end + 2..],
                None => return out,
            }
        } else {
            out.push_str("[[");
            rest = inside;
        }
    }
    out.push_str(rest);
    out
}

/// Drops the name after `{{` (up to `|` or `}}`) and every `|name=` whose name is a short run of plain words.
fn strip_names(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '{' && chars.get(index + 1) == Some(&'{') {
            out.push_str("{{");
            index += 2;
            while index < chars.len() && !matches!(chars[index], '|' | '}' | '{') {
                index += 1;
            }
            continue;
        }
        if chars[index] == '|' {
            let mut end = index + 1;
            while end < chars.len()
                && end - index <= MAX_NAME_CHARS
                && !matches!(chars[end], '=' | '|' | '[' | ']' | '{' | '}')
            {
                end += 1;
            }
            if chars.get(end) == Some(&'=') && end - index <= MAX_NAME_CHARS {
                out.push('|');
                index = end + 1;
                continue;
            }
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_names_go_but_their_values_stay() {
        assert_eq!(
            strip("{{উদ্ধৃতি ওয়েব|ইউআরএল=http://x|শিরোনাম=ঢাকার ইতিহাস|তারিখ=২০১০}}"),
            "{{|http://x|ঢাকার ইতিহাস|২০১০}}"
        );
    }

    #[test]
    fn category_and_file_links_are_dropped_but_ordinary_links_stay() {
        assert_eq!(
            strip("[[বিষয়শ্রেণী:বাংলাদেশ]] ক [[ঢাকা|রাজধানী]] খ"),
            " ক [[ঢাকা|রাজধানী]] খ"
        );
        assert_eq!(strip("[[Category:X]]text[[File:a.jpg|thumb|ছবি]]"), "text");
        assert_eq!(strip("[[ফাইল:ছবি.jpg|ছোট|ক্যাপশন]] বাক্য"), " বাক্য");
    }

    #[test]
    fn tags_and_their_attributes_go() {
        assert_eq!(strip("<ref name=\"a\">আমি</ref> তুমি"), "আমি তুমি");
        assert_eq!(strip("<!-- মন্তব্য --> কথা"), " কথা");
    }

    #[test]
    fn running_text_is_untouched() {
        let text = "আমি বাংলায় গান গাই। তুমি কোথায় = যাও?";
        assert_eq!(strip(text), text);
    }

    #[test]
    fn an_unclosed_or_overlong_name_is_left_alone() {
        assert_eq!(strip("|কখ"), "|কখ");
        let long = format!("|{}=x", "ক".repeat(60));
        assert_eq!(strip(&long), long);
    }
}
