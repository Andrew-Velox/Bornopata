# Dictionary data

The Bangla word list with frequencies behind prefix completion and ranking (`words.bnd`, built by `tools/dict-build`). **The corpora and the built `.bnd` are not committed.** This directory holds the build notes, the license record and a small hand-written sample.

## Committed files

- `sample-words.tsv`: about 170 common Bangla words with made-up counts, written for this repository. For tests and demos. No third-party data.
- `sample-corpus.txt`: five sentences written for this repository, to test corpus reading.

## Sources used for the shipped dictionary

| Source | License | Used for | Retrieved |
|---|---|---|---|
| Mozilla Common Voice text corpus, Bengali: `sentence-collector.txt` and `wiki.bn.txt` in [common-voice/common-voice](https://github.com/common-voice/common-voice) `server/data/bn` | CC0 1.0 (Mozilla's statement for the Common Voice sentence data) | word counts | see "Build record" below |
| Bengali Wikipedia, `bnwiki-latest-pages-articles.xml.bz2` from [dumps.wikimedia.org](https://dumps.wikimedia.org/bnwiki/) | CC BY-SA 4.0 (the text is also under the GFDL) | word counts | see "Build record" below |

Only words and how often they occur are kept. No sentence or running text from either source is stored or shipped.

## License of the built dictionary

Because the Wikipedia counts go in, the built `words.bnd` is distributed as **CC BY-SA 4.0**, with this attribution (also stored in the file's metadata): *Word counts derived from Bengali Wikipedia (CC BY-SA 4.0, contributors listed in the article histories) and the Mozilla Common Voice Bengali sentence corpus (CC0).*

- CC BY-SA 4.0 is one-way compatible with GPL-3.0, and the file is a separate data file next to the program, so the program's license is unchanged.
- Whether a bare frequency list is a derivative of the texts it was counted from is untested law. This record takes the conservative position.
- If only the CC0 source is used, the result carries no share-alike term (`--license CC0-1.0`).
- IndicCorp v2, OSCAR, Leipzig and mC4 are not used: their licenses are inconsistent or the underlying text rights are unresolved (see [docs/plan/word-list-sources.md](../../docs/plan/word-list-sources.md)).

## Building

```bash
cargo build --release -p borno-dict-build

# Corpora live outside the repository, for example in ~/corpora.
bzcat ~/corpora/bnwiki/bnwiki-latest-pages-articles.xml.bz2 | target/release/borno-dict-build \
    --corpus - --corpus ~/corpora/common-voice-bn \
    --min-count 5 --output words.bnd --stats \
    --name "Bornopata word list" --license CC-BY-SA-4.0 \
    --attribution "Word counts derived from Bengali Wikipedia (CC BY-SA 4.0) and the Mozilla Common Voice Bengali sentence corpus (CC0)" \
    --source "https://dumps.wikimedia.org/bnwiki/ ; https://github.com/common-voice/common-voice" --version 2026-10

install -Dm644 words.bnd ~/.local/share/bornopata/words.bnd
```

Raw wiki XML works as input: only runs of Bengali letters are used, and markup, Latin text, digits and the danda separate words. Words are normalized (NFC, ড় ঢ় য় precomposed, zero-width joiners removed except in র‍্য) and malformed spellings are dropped.

The Server and `borno-cli` use `$BORNOPATA_DICT` or `~/.local/share/bornopata/words.bnd` when present; the `upodesh` scaffold list answers only where the dictionary has no spelling of the input.
