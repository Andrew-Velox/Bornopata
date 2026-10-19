# Gloss data

The short English gloss with a part of speech shown beside a Bangla candidate (`n. rice`), for the 5,000 most frequent words of the dictionary. One gloss per word, at most two meanings in its text. The shipped file is `gloss-en.bnd`, built from `gloss-en.tsv` by `tools/gloss-build`.

## Committed files

- `gloss-en.tsv`: the table, `word<TAB>part of speech<TAB>gloss<TAB>source`, 2,454 words. The `source` column says where each gloss came from: `wiktionary` (the word's own entry), `wiktionary:<lemma>` (the word is an inflected form of that lemma and the lemma's meaning fits) or `llm:<model>`.
- `prefer-pos.tsv`: for a few words with several Wiktionary entries of different parts of speech, which one to gloss from (`ও` is mostly "and", not "he"; `হয়` is mostly the verb). It only chooses between Wiktionary's own entries; it supplies no text.
- `ATTRIBUTION.txt`: the attribution the license asks for. The build stores it in the `.bnd` metadata and the installer puts it in `share/licenses/bornopata/GLOSS-ATTRIBUTION.txt`; an About page shows it.

## Sources

| Source | License | Used for | Retrieved |
|---|---|---|---|
| English Wiktionary, Bengali entries, as the [Kaikki.org extract](https://kaikki.org/dictionary/Bengali/) (extracted 2026-10-03 from the enwiktionary dump of 2026-09-02) | CC BY-SA 4.0 and GFDL (Wiktionary's licenses; the Kaikki page does not restate them) | the glosses | 2026-10-20 |
| [Wiktextract](https://github.com/tatuylonen/wiktextract) by Tatu Ylonen, which made the extract | MIT | the extraction only; Kaikki asks to cite Ylonen, *Wiktextract: Wiktionary as Machine-Readable Structured Data*, LREC 2022 | |
| The corpus counts of the dictionary (`assets/dictionary/README.md`) | see there | which words are the most frequent | 2026-10-20 |

No language model was used for v1: `generate` exists in the tool but needs a key and none was supplied. If glosses from a model are ever added they carry `llm:<model>` in the `source` column, and the model and provider and the provider's terms are recorded here before the file ships.

## License of the built file

`gloss-en.bnd` is distributed as **CC BY-SA 4.0**, with the attribution in `ATTRIBUTION.txt` (also in the file's metadata). That matches `words.bnd`, which is CC BY-SA 4.0 because of its Wikipedia counts. The file is a separate data file next to the program, so the program's license is unchanged.

## What the cleaning does

`borno-gloss-build wiktionary` reads the Kaikki JSONL and the counts and, for each of the most frequent words:

- uses the word's own entry; letters and affixes are skipped, proper-name entries come after ordinary ones, rare, dated and dialect senses only when nothing else is left;
- drops grammar notes (`chalita infinitive of করা`, `Rarh form of …`), parentheses, articles, "to", explanations (`used to …`), numbers and Latin species names, and keeps at most two meanings of at most four words;
- for an inflected form with no usable entry of its own (and for pronoun forms) takes the meaning of the lemma, only for nouns, names, verbs and pronouns and for spelling variants, never for the forms of an adjective: genitives say `of India`, locatives `in India`, and a pronoun is put into the English case (`তার` → `his, her`);
- leaves a word without a gloss when nothing sound is left. That is half of the 5,000.

Coverage of the 5,000 (2,454 glossed: 1,781 from their own entry, 673 through a lemma):

| Most frequent | Words glossed | Their occurrences | Share of all running text |
|---|---|---|---|
| 100 | 88% | 86% | 21% |
| 1,000 | 68% | 77% | 41% |
| 5,000 | 49% | 69% | 51% |

Known limits: Wiktionary lists the senses of a word in no frequency order, so the first two meanings are sometimes not the commonest (`সাদা` shows "simple, easy", not "white"); homographs are chosen by entry order unless `prefer-pos.tsv` says otherwise; many frequent inflected forms and function words have no entry. Check a sample before trusting a gloss.

## Building

```bash
cargo build --release -p borno-gloss-build
G=target/release/borno-gloss-build

# The Kaikki extract and the corpus counts live outside the repository, for example in ~/corpora.
curl -L -o ~/corpora/wiktionary-bn/kaikki-bengali.jsonl \
    https://kaikki.org/dictionary/Bengali/kaikki.org-dictionary-Bengali.jsonl
$G wiktionary --kaikki ~/corpora/wiktionary-bn/kaikki-bengali.jsonl --counts ~/corpora/out/counts.tsv \
    --top 5000 --prefer assets/gloss/prefer-pos.tsv -o assets/gloss/gloss-en.tsv

# Optional, only with a key of your own (any OpenAI-compatible endpoint with a free tier). The key is read from the
# environment or an ignored .env file and is never printed or committed:
#   BORNO_LLM_API_KEY=...  BORNO_LLM_BASE_URL=https://.../v1  BORNO_LLM_MODEL=...
$G generate --counts ~/corpora/out/counts.tsv --entries assets/gloss/gloss-en.tsv -o data/generated/gloss-llm.tsv

# The review sheet of the 200 most frequent words, next to what Wiktionary says:
$G sheet --entries assets/gloss/gloss-en.tsv --counts ~/corpora/out/counts.tsv \
    --kaikki ~/corpora/wiktionary-bn/kaikki-bengali.jsonl --rows 200 -o data/generated/gloss-review-200.tsv

# The file the input method loads:
$G export --entries assets/gloss/gloss-en.tsv -o gloss-en.bnd --license CC-BY-SA-4.0 \
    --attribution "$(cat assets/gloss/ATTRIBUTION.txt)" --source https://kaikki.org/dictionary/Bengali/ \
    --version 2026-10 --name "Bornopata English glosses"
```

Put `gloss-en.bnd` in `~/.local/share/bornopata/` (`apps/linux/scripts/install.sh` does it) and keep `[general] gloss = true`.
