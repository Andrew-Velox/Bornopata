---
title: What it learns
order: 1
description: Bornopata remembers the words you choose and the order you write them, only on your computer. How it decays, how to forget a word, and how to switch it off or reset it.
---

Bornopata gets better the more you use it. It remembers three things, all on your own computer:

1. **Which word you chose for which letters.** After you pick `কোথা` for `kotha`, it comes first the next time you type `kotha`. A word that is in no dictionary is learned the same way, so your own vocabulary (names, slang, new words) grows.
2. **How often you use each word.** Common words for you rank a little higher everywhere.
3. **Which word usually follows which.** This drives the [next-word suggestions](next-words.md) and nudges the order of candidates after the previous word.

## What is not learned

- [Your own phrases](../input/phrases.md), and dates and times from [`tarikh`, `somoy` and `bar`](../input/shortcuts.md). Both can be private or only true today.
- Anything typed in a **password field**: there Bornopata does nothing at all.
- Anything typed in a field an application marks as **sensitive**: Bornopata still converts your typing but remembers nothing from it.
- Anything, if you turn learning off.

## It fades and it is kept small

Learning cools over time: a word you stopped using loses weight with a half-life of about three months, and entries not used for a year are dropped. The personal word-order table is capped, and the least-used pairs are thinned when it grows past 200,000.

## Forget a word

Chose the wrong word once and it keeps coming first? Type its letters, highlight the unwanted candidate and press **`Shift+Delete`**. Bornopata removes your choice of that word for those letters, and its place in your word-order table when no other letters still lead to it. Words that come from the dictionary or the Avro rules stay in the list: only what you taught is removed.

On a [next-word suggestion](next-words.md) the same key stops that suggestion being offered.

## Switch it off, or start again

To stop learning, set this in `~/.config/bornopata/config.toml` (the Server picks it up by itself):

```toml
[general]
learning = false
```

To erase everything it learned, stop the Server (`systemctl --user stop bornopata`), delete `user-words.tsv`, `user-choices.tsv` and `user-ngram.tsv` from `~/.local/share/bornopata/`, and start it again. The files are plain text, so you can also read or edit them. See [Your files and your privacy](../help/data-and-files.md).

Learned data is saved a few seconds after you stop typing, and when the Server stops.
