# Bangla engine design

Decided 2026-10-20. Replaces the inherited pinyin pipeline in `borno-core`. Reuse decisions are in [bangla-reuse-plan.md](bangla-reuse-plan.md).

## What the building blocks do

- **okkhor** (`Parser::new_phonetic().convert`): deterministic Avro conversion, one spelling per input. Only `o i u d g j n r s t y z` keep their case (`t`/`T`, `d`/`D`, `n`/`N`, `o`/`O` are different letters); every other letter is lowercased first.
  Conjuncts come from fixed rules (`kkh` gives ক্ষ). Digits become Bangla digits, punctuation and spaces pass through, so word splitting is the Core's job.
- **upodesh** (`avro::Suggest::suggest`): whole-word phonetic variants that exist in its list, in pattern order, not frequency order. No prefix completion. Single letters return noise.
  Its list has unverified provenance, so it stays a development scaffold behind `WordSource`.

## Composition

- The buffer holds raw Roman letters and **keeps case** (Avro semantics); there is no syllable segmentation and no Shift-lowercasing.
- Composing characters are ASCII letters, plus `` ` ``, `^` and `:` once the buffer is non-empty (they are Avro pattern characters: candrabindu, visarga, the cancel mark).
- Any other character ends the word: the top (or highlighted) candidate is committed, then the character is converted (`.` gives ।, digits give Bangla digits, space stays a space) and emitted with it.

## The dictionary

`borno-dictionary` holds the word list: about 413k Bengali words with corpus counts, built by `tools/dict-build` (see [assets/dictionary/README.md](../../assets/dictionary/README.md)) into one `.bnd` file that is mapped, not loaded.

- **Words** are normalized so a typed word and its entry are equal strings: NFC, then ড় ঢ় য় precomposed (as `okkhor` writes them), ZWNJ removed, ZWJ kept only in র‍্য, malformed spellings dropped.
- **The phonetic key** (`borno_dictionary::key`) reduces a spelling to what people type alike. `okkhor` makes one Bangla spelling of the typed Roman; the key of that spelling is looked up. Confusable letters share a class ({শ ষ স}, {ন ণ}, {জ য}, {ত ট}, {থ ঠ}, {দ ড}, {ধ ঢ}, {র ড় ঢ়}, {ঙ ং}), vowel length and independent versus dependent vowels are ignored, the o sign, the hasanta, the y-phala and ba-phala, the joiners and the nasalization mark vanish, and runs of one consonant collapse. So `kotha`, `kOtha` and `kothaa` all find কথা and কোথা, and `bishshobiddaloy` finds বিশ্ববিদ্যালয়. The key of a prefix is a prefix of the key, which is what makes prefix completion work.
- **Variant spellings** (`borno_dictionary::variant_keys`) reach words the typing cannot: `moddhe` for মধ্যে (দ্ধ read as ধ), `porjonto` for পর্যন্ত and `surjo` for সূর্য (a জ right after র dropped), `dukkho` for দুঃখ (ক্ষ read as ঃখ) and `chai` for চাই (চ and ছ swapped). The source looks up the key of each variant besides the main key and adds what it finds with its frequency divided by 16, so the words the typing itself reaches come first and a variant wins only when it is far more common. They are query-side on purpose: the key is stored in `words.bnd`, so changing `key` means rebuilding and re-releasing the dictionary, while a variant rule only needs a new build of the program.
- **Index:** an FST from key to a group of word ids. Ids are numbered by falling frequency, so every id list in ascending order is already ranked. Keys up to three characters have their 16 best words precomputed; longer prefixes stream the matching keys and keep the best few. A lookup takes 10-40 µs; opening validates the file in about 50 ms.
- **Seam:** `DictionarySource` implements `WordSource::lookup`, which returns each word with its frequency and whether it is a spelling of the input or only a completion. `FallbackWordSource` asks the `upodesh` scaffold only when the dictionary has no spelling of the input.

## Candidates

For a word `w`:

1. **direct**: okkhor's conversion of `w`. Always present when non-empty, even if no dictionary word matches.
2. **dictionary**: `WordSource::lookup(w)` words that spell `w`, with their frequencies.
3. **completion**: words that only start with `w` (the key continues).
4. **learned**: words the user chose before for exactly this input, from the learner, even when none of the above produces them.

Deduplicated by text. A candidate's source label is learned, then direct, then dictionary, then completion, whichever applies first.

## Ranking

Spellings of the input (learned, direct, dictionary) are ranked by score; completions follow them whatever their score, ranked among themselves by the same score. So the first candidate, which Space commits, is always something the user typed, and a frequent completion can be picked with its number or the arrow keys but never commits by surprise. Ties break on the text so the order is stable.

```
score = prior − gap + 0.6·log10(1 + frequency) + 4·ln(1 + choice_weight(input, text))
        + 0.2·ln(1 + weight(text)) + 0.5·ln(1 + pair_count(previous, text)) + lm_bonus
prior:    direct and in the dictionary 1.7, dictionary only 1.5, direct only 1.0, completion 1.0
gap:      0.1 per letter of edit distance from the direct conversion, dictionary-only spellings, at most 0.5
lm_bonus: 0.2 · max(0, log_prob(previous, text) + 10) when the language model knows the pair
```

On `assets/dictionary/eval-spellings.tsv` (142 typical spellings) with the shipped dictionary this puts the right word first 92% of the time and within the first nine 96.5% of the time. The weights were tuned on that set, so treat the numbers as optimistic; retune with `borno-cli --eval` when there is real typing data.

- A dictionary word that is much more frequent than the direct conversion wins (`kothay` gives কোথায় before কথায় once the counts say so); about as frequent keeps the typed form first.
- The weight of a word committed for some other input is deliberately small: after `ta` was committed, `Ta` must still lead with টা. Only a choice made for exactly this input outweighs the typed letters, and one choice is worth about a thousandfold frequency gap.

## Learning and prediction

- `Learner` (trait in `borno-core`, implemented by `borno-learning`) records what the user committed: per-input choices, plain word counts, and the personal bigram/trigram (`UserNgram`). Choices decay with time.
- `LanguageModel` (trait in `borno-core`, implemented by `borno-lm`) scores `log P(word | previous)` and lists likely successors for next-word suggestions.
- Next-word suggestions after a commit merge the personal n-gram followers and the language model successors. They are separate from candidates and shown only when the buffer is empty.

## Input before learning

Candidate generation is synchronous, in memory and reads only local tables. Nothing here waits on a network call, a disk write or a learning update. Recording a commit happens after the text has been returned.

## Out of scope for this phase

Cloud prediction, glosses, neural rescoring, emoji and fixed layouts. The crates for the first three were dropped; the Server, config and protocol are described in [../notes/linux-fcitx5.md](../notes/linux-fcitx5.md).
