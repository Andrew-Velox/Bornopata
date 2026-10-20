# assets

Bundled data sources and samples. Every subdirectory has a README that states where the data came from and under which license.

## Directories

- `dictionary/`: the sample, the evaluation spellings and the build and license record of the word list (the corpora and the built `words.bnd` are not committed).
- `gloss/`: the English glosses beside the candidates: the table, the license and attribution record, and how it is built (from Wiktionary; the built `.bnd` is not committed).
- `emoji/`: the emoji beside the words: the table (Unicode CLDR names and keywords, Unicode License v3), its license text and how it is built.
- `sample/`: tiny hand-written files from the inherited pinyin tooling, still used by a few tests.

## Data of unverified provenance

The word list shipped with `upodesh`, which is the fallback where the dictionary has no spelling of the input, (and `riti`'s `dictionary.json` and `autocorrect.json`) has no stated origin or license. Until the maintainers answer
(draft question: [docs/notes/upodesh-provenance-inquiry.md](../docs/notes/upodesh-provenance-inquiry.md)), treat it as a **development scaffold only: provenance unverified, must be replaced before release**.
Any directory that carries or derives from it must say so at the top of its README. Our own list will come from a redistributable corpus; see [docs/plan/word-list-sources.md](../docs/plan/word-list-sources.md).
