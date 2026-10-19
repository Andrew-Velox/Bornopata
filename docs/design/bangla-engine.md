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

## Candidates

For a word `w`:

1. **direct**: okkhor's conversion of `w`. Always present when non-empty, even if no dictionary word matches.
2. **dictionary**: `WordSource::suggest(w)` in source order.
3. **learned**: words the user chose before for exactly this input, from the learner, even when neither of the above produces them.

Deduplicated by text. A candidate's source label is learned, then direct, then dictionary, whichever applies first.

## Ranking

Score per candidate, highest first, ties broken on the text so the order is stable:

```
score = prior + 3·ln(1 + choice_weight(input, text)) + 0.2·ln(1 + weight(text))
        + 0.5·ln(1 + pair_count(previous, text)) + lm_bonus
prior:    direct and in the dictionary 2.0, dictionary only 1.5, direct only 1.0
lm_bonus: 0.2 · max(0, log_prob(previous, text) + 10) when the language model knows the pair
```

A dictionary word beats an unbacked direct conversion (`ShikkhA` gives শিক্ষা first and ষিক্ষা stays visible). The weight of a word committed for some other input is deliberately small:
after `ta` was committed, `Ta` must still lead with টা. Only a choice made for exactly this input outweighs the typed letters.

The constants are first guesses to be tuned with the CLI once there is real frequency data. The scaffold list has no frequencies and `upodesh` answers in a different order every run (the adapter sorts its words), so position in the word source is not used yet: direct, dictionary membership and learning do the work.

## Learning and prediction

- `Learner` (trait in `borno-core`, implemented by `borno-learning`) records what the user committed: per-input choices, plain word counts, and the personal bigram/trigram (`UserNgram`). Choices decay with time.
- `LanguageModel` (trait in `borno-core`, implemented by `borno-lm`) scores `log P(word | previous)` and lists likely successors for next-word suggestions.
- Next-word suggestions after a commit merge the personal n-gram followers and the language model successors. They are separate from candidates and shown only when the buffer is empty.

## Input before learning

Candidate generation is synchronous, in memory and reads only local tables. Nothing here waits on a network call, a disk write or a learning update. Recording a commit happens after the text has been returned.

## Out of scope for this phase

Cloud prediction, glosses, neural rescoring, emoji, fixed layouts, config files and the Server protocol. Their crates are excluded from the workspace until the next phase ports them.
