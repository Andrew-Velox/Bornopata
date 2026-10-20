---
title: The candidate list
order: 2
description: Where candidates come from (your letters, the dictionary, completions, your choices), how they are ordered, and what the marks in the window mean.
---

Every time you type a letter, Bornopata builds a list of up to nine candidates for the word so far. Nothing is sent anywhere; the list is made on your computer in a fraction of a millisecond.

## Where candidates come from

1. **Your own phrases** for exactly this code come first; see [Your own phrases](phrases.md).
2. **The Avro conversion** of exactly what you typed. This is always in the list, so any word can be typed even if it is in no dictionary.
3. **Dictionary words** spelled like your input, or close to it. The dictionary has about 413,000 words with how often each occurs, so common words rank higher. A small built-in word list fills in spellings the dictionary does not have.
4. **Words you chose before** for this input: they move up.
5. **Completions**: longer words that *start* with what you typed. They are always listed after the real spellings of your input.
6. **Extras**: the date, time and weekday for `tarikh`, `somoy` and `bar` ([Date, time and weekday](shortcuts.md)), and emoji ([Emoji](emoji.md)).

## How they are ordered

The order combines how well a word matches the letters you typed, how common it is, what you chose before for this input, and what you usually type after the previous word. A word you have chosen for an input usually comes first the next time you type it.

Different Roman spellings of one sound end up at the same words: `bhalobasha`, `valobasha` and `bhalobasa` all offer `ভালোবাসা` and `ভালবাসা` at the top, in a slightly different order.

## What the window shows

In Bornopata's own window:

- `★` before a candidate: a word you chose before for this input (or one of your own phrases).
- `…` after a candidate: a completion.
- A short grey meaning after a candidate (`n. rice`): its English gloss; see [English glosses](../learning/glosses.md).
- `→` before a word: a next-word suggestion, shown after a space.

Where Fcitx5's own panel is used instead, the same information appears in its comment column.

## Choosing

Use `Space` for the first candidate, a digit for another, or the arrow keys then `Space`. See [Keys and shortcuts](../getting-started/keys.md).
