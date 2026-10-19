//! Writing the `.bnd` file from gloss tables.

use std::path::Path;

use borno_format::Metadata;
use borno_gloss::GlossaryBuilder;

use crate::error::BuildError;
use crate::table::Entry;

/// Builds the glossary file from `entries`; a word that appears twice keeps its last entry.
pub fn export(entries: &[Entry], output: &Path, metadata: &Metadata) -> Result<usize, BuildError> {
    if entries.is_empty() {
        return Err(BuildError::Empty);
    }
    let mut builder = GlossaryBuilder::new();
    for entry in entries {
        builder.add(&entry.word, &entry.pos, &entry.text)?;
    }
    builder.write(output, metadata)?;
    Ok(builder.len())
}

#[cfg(test)]
mod tests {
    use borno_gloss::Glossary;

    use super::*;

    #[test]
    fn the_exported_file_answers_for_every_entry_and_refuses_an_empty_table() {
        let path =
            std::env::temp_dir().join(format!("borno-gloss-export-{}.bnd", std::process::id()));
        let entries = [
            Entry {
                word: "ভাত".into(),
                pos: "n.".into(),
                text: "rice".into(),
                source: "wiktionary".into(),
            },
            Entry {
                word: "যাওয়া".into(),
                pos: "v.".into(),
                text: "go".into(),
                source: "wiktionary".into(),
            },
        ];
        let metadata = Metadata {
            name: "test".into(),
            license: "CC-BY-SA-4.0".into(),
            ..Metadata::default()
        };
        assert_eq!(export(&entries, &path, &metadata).unwrap(), 2);
        let glossary = Glossary::open(&path).unwrap();
        assert_eq!(glossary.lookup("ভাত").unwrap().text, "rice");
        assert_eq!(glossary.lookup("যাওয়া").unwrap().pos, "v.");
        assert!(matches!(
            export(&[], &path, &metadata),
            Err(BuildError::Empty)
        ));
        std::fs::remove_file(path).unwrap();
    }
}
