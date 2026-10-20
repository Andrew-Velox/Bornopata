# Dictionary data

The Bangla word list with frequencies behind prefix completion and ranking (`words.bnd`, built by `tools/dict-build`). **The corpora and the built `.bnd` are not committed.** This directory holds the build notes, the license record and a small hand-written sample.

## Committed files

- `sample-words.tsv`: about 170 common Bangla words with made-up counts, written for this repository. For tests and demos. No third-party data.
- `sample-corpus.txt`: five sentences written for this repository, to test corpus reading.
- `eval-spellings.tsv`: 142 common words with the Avro-style spellings people type for them, written for this repository. `borno-cli --eval` measures with it how often the right word is among the first candidates.

## Sources used for the shipped dictionary

| Source | License | Used for | Retrieved |
|---|---|---|---|
| Mozilla Common Voice text corpus, Bengali: `sentence-collector.txt` and `wiki.bn.txt` in [common-voice/common-voice](https://github.com/common-voice/common-voice) `server/data/bn` | CC0 1.0 (Mozilla's statement for the Common Voice sentence data) | word counts | 2026-10-20 |
| Bengali Wikipedia, `bnwiki-latest-pages-articles.xml.bz2` from [dumps.wikimedia.org](https://dumps.wikimedia.org/bnwiki/) | CC BY-SA 4.0 (the text is also under the GFDL) | word counts | dump of 2026-10-01, retrieved 2026-10-20 |

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
    --wikitext --corpus - --corpus ~/corpora/common-voice-bn \
    --min-count 5 --output words.bnd --stats \
    --name "Bornopata word list" --license CC-BY-SA-4.0 \
    --attribution "Word counts derived from Bengali Wikipedia (CC BY-SA 4.0) and the Mozilla Common Voice Bengali sentence corpus (CC0)" \
    --source "https://dumps.wikimedia.org/bnwiki/ ; https://github.com/common-voice/common-voice" --version 2026-10

install -Dm644 words.bnd ~/.local/share/bornopata/words.bnd
```

Raw wiki XML works as input: only runs of Bengali letters are used, and markup, Latin text, digits and the danda separate words. Words are normalized (NFC, ড় ঢ় য় precomposed, zero-width joiners removed except in র‍্য) and malformed spellings are dropped.

## Build record

The dictionary measured below was built on 2026-10-20 with `--wikitext --min-count 5`:

| Input | SHA-256 |
|---|---|
| `common-voice-bn/sentence-collector.txt` (432,697 lines) | `478e389b68dce47a968ae8ff45841882d2a83e2baf70963bbea9a42aba17b7dd` |
| `common-voice-bn/wiki.bn.txt` (129,935 lines) | `e43892b11be9dc1c5a414cd581b32daf6bf273076cfa41421a42a541f85d68df` |
| `bnwiki-latest-pages-articles.xml.bz2` (dump of 2026-10-01) | `f32f68a13fb486423b4cebea3ac79cbc16c6ee2f2e138045aba6795827b39c14` |

113.2 million Bengali tokens (0.6 million malformed ones dropped), 1.94 million distinct words, **413,184 kept**, a 20.6 MB `words.bnd`. The top 10,000 words cover 82% of the kept occurrences. Building takes about 100 seconds; `--dump-counts` saves the kept counts so a later `--counts` build (to try another phonetic key, say) takes seconds.

Without `--wikitext` the template and parameter names of the dump (`|তারিখ=`, `বিষয়শ্রেণী`, `ইউআরএল`) were among the most frequent words.

`borno-cli --dict words.bnd --eval assets/dictionary/eval-spellings.tsv` on this build: the right word is first for 92.3% of the 142 spellings, among the first three for 96.5%, and among the first nine for 96.5%. The ranking weights were tuned on this very set, so the numbers are optimistic for other spellings. Start-up of the Server with it is about 50 ms; a lookup takes 10-40 µs.

The Server and `borno-cli` use `$BORNOPATA_DICT` or `~/.local/share/bornopata/words.bnd` when present; the `upodesh` scaffold list answers only where the dictionary has no spelling of the input.

## Word pair model

`lm.bnd` (built by `tools/lm-build`) counts which word follows which. It gives next-word suggestions from the first day and lets the previous word decide between spellings. It is a data file next to the dictionary; the corpora are not committed.

- Source: the Mozilla Common Voice Bengali sentence corpus (`sentence-collector.txt` and `wiki.bn.txt`, the same two files as above; CC0 by Mozilla's statement, the `wiki.bn.txt` sentences come from Wikipedia, CC BY-SA 4.0). The Wikipedia dump itself was tried and rejected: its talk and project pages put boilerplate (`ভালো নিবন্ধ`, `ধন্যবাদ ব্যবহারকারী`) in the suggestions, and its formal register ranks everyday spellings worse than the sentence corpus does.
- License: **CC BY-SA 4.0**, the same conservative position as the dictionary. Only words and pair counts are kept, no sentences.
- Words: the first 30,000 of the dictionary's counts (`--dump-counts`), so both files spell alike. A word outside them breaks the pair chain.
- Kept: pairs seen at least 3 times, the 16 likeliest successors of each word.

```bash
target/release/borno-dict-build ... --dump-counts counts.tsv   # as above, once
cargo build --release -p borno-lm-build
target/release/borno-lm-build --counts counts.tsv \
    --corpus ~/corpora/common-voice-bn/sentence-collector.txt --corpus ~/corpora/common-voice-bn/wiki.bn.txt \
    --min-count 3 --output lm.bnd --license CC-BY-SA-4.0 \
    --attribution "Word pair counts derived from the Mozilla Common Voice Bengali sentence corpus (CC0), whose Wikipedia sentences are CC BY-SA 4.0" \
    --source "https://github.com/common-voice/common-voice" --version 2026-10
```

Build record of the pinned file (`lm.bnd.release`): 28,626 words, 88,156 pairs, 2.0 MB, built in about a second. Measured with `borno-cli --eval`: top-1 of the hand-written set rose from 94.4% to 95.8%, and with the previous word `tumi kothay thako` gives কোথায় and `kono` gives কোনো.
