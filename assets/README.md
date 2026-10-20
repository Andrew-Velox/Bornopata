# assets

Bundled data sources and samples. Every subdirectory has a README that states where the data came from and under which license.

## Directories

- `dictionary/`: the sample, the evaluation spellings and the build and license record of the word list (the corpora and the built `words.bnd` are not committed).
- `gloss/`: the English glosses beside the candidates: the table, the license and attribution record, and how it is built (from Wiktionary; the built `.bnd` is not committed).
- `emoji/`: the emoji beside the words: the table (Unicode CLDR names and keywords, Unicode License v3), its license text and how it is built.
- `sample/`: tiny hand-written files from the inherited pinyin tooling, still used by a few tests.

## The `upodesh` word list

The word list shipped with `upodesh` is the fallback where our dictionary has no spelling of the input. `upodesh` is MIT and its list is the one of `libavrophonetic` (MIT as well); neither says where the words first came from. The project owner decided on 2026-10-20 to use it under its MIT license anyway. The reasoning, what was checked and the conditions (swappable behind `WordSource`, MIT notices shipped with releases) are in [docs/design/bangla-reuse-plan.md](../docs/design/bangla-reuse-plan.md). `riti`'s `dictionary.json` and `autocorrect.json` are not used. Data of unverified provenance that is not covered by such a decision is still a development scaffold and must be marked so at the top of its README. Our own list comes from redistributable corpora; see [docs/plan/word-list-sources.md](../docs/plan/word-list-sources.md).
