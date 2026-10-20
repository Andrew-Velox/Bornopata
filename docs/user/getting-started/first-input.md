---
title: Your first words
order: 2
description: Type a word in Roman letters, pick the Bangla spelling you want, commit it with Space, and correct a wrong choice.
---

Switch to Bornopata and start typing in any text field.

## Type a word

Type the word the way it sounds, in Roman letters: `ami`. As you type, the letters are shown as underlined Bangla in your text (`আমি`) and a window lists the candidates:

```text
ami
1   আমি
2   আমিই
3   আম্মি
4   আমিরাত…
```

- The first candidate is the one `Space` will type.
- In Bornopata's own window, a `★` marks a word you chose before. A trailing `…` marks a word that only *starts* with what you typed (a completion), which is always listed after the real spellings of your input.
- The list shows at most nine candidates.

## Choose

| You want | Press |
|---|---|
| The first candidate, then a space | `Space` |
| The first candidate and a new line (or whatever Enter does in that field) | `Enter` |
| Candidate number 3 | `3` (the digits `1` to `9`) |
| Another candidate with the keyboard | `↓` / `↑` (or `Tab` / `Shift+Tab`) to move, then `Space` |

Punctuation also ends the word: typing `.` after `ami` gives `আমি।` (the Bangla full stop `।`).

## Correct

- `Backspace` removes the last Roman letter and the list updates.
- `Esc` throws the word away.
- Chose the wrong word and it keeps coming first? Highlight it again and press `Shift+Delete` to make Bornopata forget it; see [What it learns](../learning/learning.md).

## After a space: the next word

When you press `Space`, Bornopata may offer a few words that usually follow the one you just typed. Press `Tab` to take the highlighted one, `↓` / `↑` to choose another, or simply keep typing to ignore them. See [Next-word suggestions](../learning/next-words.md).

## Switch to English

To type plain English without leaving the input method, press `Ctrl+Alt+Space`. Press it again to return to Bangla. See [English mode](../input/english-mode.md).
