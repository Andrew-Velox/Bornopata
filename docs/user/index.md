---
title: Bornopata user guide
order: 0
description: Bornopata is a free Bangla phonetic input method for Linux. Type Bangla with Roman letters, and it learns your words on your own computer.
---

Bornopata (বর্ণপাতা) is a Bangla input method. You type Bangla the way you say it, in Roman letters (`ami bhat khai`), and it offers the Bangla words, learns the ones you choose, and suggests what you are likely to type next.

It is a preview for **Linux with Fcitx5**. Windows and macOS versions are planned.

## Where to start

- Not installed yet: [Install](getting-started/install.md).
- Installed and ready to type: [Your first words](getting-started/first-input.md).
- Looking for a key: [Keys and shortcuts](getting-started/keys.md).
- How to spell a Bangla letter in Roman: [Typing Bangla](input/typing-bangla.md).
- Something does not work: [Troubleshooting](help/troubleshooting.md).

## What it does

- **Phonetic typing.** The spelling you type is converted by the Avro phonetic rules, and a dictionary of 413,000 words offers the real words near it. Different Roman spellings of the same sound (`bhalobasha`, `valobasha`) tend to find the same word.
- **A candidate window at your cursor.** In Wayland applications such as Brave, Firefox, GTK and Qt programs the window appears at the text cursor, in one of five themes.
- **It learns from you.** Words you choose come first next time, and the next-word suggestions follow what you actually write. All of it stays on your machine.
- **English beside the Bangla.** A short English meaning (`n. rice`) can show beside a candidate, so you pick up English words as you type.
- **Your own phrases, today's date, emoji.** `mymail` can expand to your address, `tarikh` offers today's date, and `hridoy` offers a heart.

## Three principles

- **Typing comes first.** Nothing that is added for a secondary feature may slow typing down or get in the way of it.
- **Everything stays on your machine.** Typing, the dictionary, learning and the glosses never use the network, and the typed text is never written to a log. See [Your files and your privacy](help/data-and-files.md).
- **It is free software.** The program is GPL-3.0-or-later; the dictionary and glosses are CC BY-SA 4.0.
