# English glosses beside the candidates

Decided and built 2026-10-20 for Linux ("learn while you type", as Qingjian had it). Each Bangla candidate can show one short English gloss with a part of speech, such as `n. rice`, beside the word.

## What it is

- **One gloss per word, one auxiliary language.** A part of speech and a text with at most two meanings (`n. group, team`). Never a list of languages: the product learns one language at a time (see [../contributing.md](../contributing.md)). The data file is `gloss-en.bnd`; another language would be another file, chosen by one setting.
- **Input comes first.** Candidates are generated and ranked without it. The gloss is looked up afterwards, when the frame is built (`Engine::annotations()`), from a memory-mapped hash table: one probe per candidate, nine per frame, so nothing waits and nothing is asynchronous. A word without an entry has no gloss and the row is as before.
- **Setting:** `[general] gloss = true|false`. Off (or no file) means the Server never maps the file and no frame carries a gloss.

## Parts

- `borno-core`: the `Annotator` trait and `Annotation { pos, text }`; `Engine::with_annotator`, `Engine::annotations()` (for the candidates) and `Engine::annotate(word)` (for next-word suggestions). Ranking never reads it.
- `borno-gloss`: `Glossary` maps a `.bnd` of kind glossary (text arena, 20-byte entry records, hash index over the words, validated on open) and implements `Annotator`; `GlossaryBuilder` writes it. Words are stored in the dictionary's canonical spelling, so any spelling finds its gloss.
- `tools/gloss-build`: `wiktionary` (clean Wiktionary into the table), `generate` (fill gaps with a language model, optional), `sheet` (the review sheet), `export` (table to `.bnd`).
- Server: `Shared::with_glossary`, `Session::frame` puts `gloss: {pos, text}` on each `FrameCandidate`. The field is optional and skipped when absent, so the protocol stays at version 4 and an older plugin ignores it. `render_frame` turns it into the row's annotation (`Tone::Pos`, `Tone::Gloss`), which the themes already draw; the Fcitx5 plugin shows it as the candidate comment (`n. rice`, after the star of a learned word).

## Where the glosses come from

The first version covers the **5,000 most frequent words**, from **English Wiktionary** (the Kaikki extract), cleaned by rules, not rewritten by hand. The alternatives were measured on the 100-word sample of 2026-10-20 and in the 5,000: Wiktionary has an entry for 50% of the top 5,000 (17% of the top 50,000); a language model covers everything but must be checked and needs a key and money, which this version does without. The model step exists (`generate`, free-tier, key only from the environment or `.env`) and is not part of the shipped data. Sources, licenses and numbers: [../../assets/gloss/README.md](../../assets/gloss/README.md).

The cleaning keeps a gloss only when it is a *sound* one: the word's own entry first; for an inflected form the lemma's meaning, but only for nouns, names, verbs, pronouns and spelling variants (never for adjective forms), with `of` for the genitive, `in` for the locative and English case for pronouns (`তার` → `his, her`). Grammar notes ("chalita infinitive of X"), parentheses, articles, explanations, numbers and species names are removed, and at most two meanings of at most four words are kept. A word with nothing sound left has no gloss, which is half of the 5,000.

## License

CC BY-SA 4.0 (Wiktionary's), like `words.bnd`. The attribution text is stored in the file's metadata, installed to `share/licenses/bornopata/GLOSS-ATTRIBUTION.txt` and printed by the installer; an About page shows it from the metadata.

## Limits

- Wiktionary lists senses in no frequency order, so the first two meanings are sometimes not the commonest. `assets/gloss/prefer-pos.tsv` picks the entry for a few homographs; every other choice needs a person to review the sheet.
- Many frequent forms and function words have no Wiktionary entry and stay without a gloss until a model run (with the owner's key) or a reviewed list fills them.
- The window gets wider with a gloss; a theme can drop the annotation node. There is no per-row switch.
