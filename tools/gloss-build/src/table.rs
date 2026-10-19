//! The gloss table as a TSV file: `word<TAB>part of speech<TAB>text<TAB>source`, sorted by word.

use std::path::Path;

use crate::error::BuildError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub word: String,

    /// `n.`, `v.`…; may be empty.
    pub pos: String,

    pub text: String,

    /// Where the gloss came from: `wiktionary`, `wiktionary:<lemma>` or `llm:<model>`.
    pub source: String,
}

/// Lines starting with `#` are comments, so the file can carry its license and attribution.
pub fn read(path: &Path) -> Result<Vec<Entry>, BuildError> {
    let text = std::fs::read_to_string(path).map_err(|source| BuildError::Read {
        path: path.to_owned(),
        source,
    })?;
    let mut entries = Vec::new();
    for (number, raw) in text.lines().enumerate() {
        if raw.trim().is_empty() || raw.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = raw.split('\t').collect();
        let [word, pos, text, source] = fields[..] else {
            return Err(BuildError::Line {
                path: path.to_owned(),
                line: number + 1,
                reason: "expected word, part of speech, text and source, separated by tabs"
                    .to_owned(),
            });
        };
        entries.push(Entry {
            word: word.to_owned(),
            pos: pos.to_owned(),
            text: text.to_owned(),
            source: source.to_owned(),
        });
    }
    Ok(entries)
}

/// Writes `entries` sorted by word, after `header` (written as comment lines).
pub fn write(path: &Path, header: &str, entries: &[Entry]) -> Result<(), BuildError> {
    let mut sorted: Vec<&Entry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.word.cmp(&b.word));
    let mut out = String::new();
    for line in header.lines() {
        out.push_str("# ");
        out.push_str(line);
        out.push('\n');
    }
    for entry in sorted {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            entry.word, entry.pos, entry.text, entry.source
        ));
    }
    std::fs::write(path, out).map_err(|source| BuildError::Write {
        path: path.to_owned(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(word: &str, text: &str) -> Entry {
        Entry {
            word: word.into(),
            pos: "n.".into(),
            text: text.into(),
            source: "wiktionary".into(),
        }
    }

    #[test]
    fn a_written_table_reads_back_sorted_without_its_header() {
        let path =
            std::env::temp_dir().join(format!("borno-gloss-table-{}.tsv", std::process::id()));
        write(
            &path,
            "License: test\nSecond line",
            &[entry("খ", "b"), entry("ক", "a")],
        )
        .unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("# License: test\n# Second line\n"));
        assert_eq!(read(&path).unwrap(), [entry("ক", "a"), entry("খ", "b")]);
        std::fs::write(&path, "ক\tn.\ta\n").unwrap();
        assert!(read(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
}
