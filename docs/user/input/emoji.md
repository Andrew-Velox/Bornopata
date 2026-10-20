---
title: Emoji
order: 5
description: Bornopata offers emoji next to the Bangla words and English words that name them, for example a heart for hridoy or heart.
---

Emoji appear in the candidate list right after the word that names them.

```text
hridoy
1 হৃদয়
2 🫀  (the name of the emoji shows beside it)
3 ❤️
4 হৃদ্য
```

## How they are found

- **By a Bangla word.** The best-ranked candidate that has emoji gets up to two after it: `hridoy` gives `হৃদয়` then `🫀` and `❤️`; `taka` gives `টাকা` then `🤑` and `💱`.
- **By an English word typed in Roman letters.** In lower case and at least three letters, the English name works too: `heart` shows the Avro conversion first and then `❤️` and another heart. Shorter words are not looked up, because too many of them are also Bangla words.
- At most **three** emoji in a list, so the words keep most of the window. The emoji's name (in Bangla, otherwise English) shows beside it where an English gloss would.
- `Space` always types the first candidate, never an emoji, unless you have chosen that emoji for that input before.

## Learning

Choosing an emoji is learned like any word, for the letters you typed: after you pick `🫀` for `hridoy` once, it comes first for `hridoy` from then on. `Shift+Delete` on it makes Bornopata forget.

## Where the emoji come from

The names and keywords are the Unicode CLDR annotations in Bangla and English (Emoji up to version 15.1; newer emoji do not draw in common fonts yet). Because they are CLDR's own words, a spelling that differs from CLDR's (`ভালবাসা` against `ভালোবাসা`) may find a different emoji, or none.

To show them in the candidate window you need a color emoji font, such as Noto Color Emoji.

To turn emoji off, set `emoji = false` in `[general]`; see [The config file](../settings/config-file.md).
