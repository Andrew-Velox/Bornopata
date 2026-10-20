---
title: Next-word suggestions
order: 2
description: After a space Bornopata offers up to five words that usually follow the one you just typed. Take one with Tab.
---

After you commit a word with `Space`, the window may show a few words that usually come next (the words here are only an example):

```text
ami →
→ ভাত
→ যাই
→ ভালো
```

They come from two places, **your own writing** first: the words you have typed after this word before, most frequent first. Then a small built-in model of which words usually follow which, made from Bengali text, so there are suggestions from the first day. As Bornopata learns your habits, your own words move to the front. Remove a suggestion you do not want with `Shift+Delete` and it is not offered after that word again.

| Key | What it does |
|---|---|
| `Tab` | Types the highlighted suggestion (it is followed by its own suggestions) |
| `↓` / `↑` | Moves the highlight |
| `Shift+Delete` | Stops this suggestion from being offered again |
| `Esc` | Hides the suggestions |
| Any other key | Hides them and does its normal job: just keep typing |

Suggestions never get in the way of typing: the arrow keys only move through them while they are shown, and a letter simply starts the next word.

Choosing a suggestion is learned too, so the habits it reflects keep adjusting. To stop learning altogether, see [What it learns](learning.md).
