---
title: Keys and shortcuts
order: 3
description: Every key Bornopata handles while you type, choose, correct and switch modes, and which shortcuts you can change.
---

## While typing a word

| Key | What it does |
|---|---|
| `a`–`z`, `A`–`Z` | Adds a letter to the word. Capitals are different letters (`t` is `ত`, `T` is `ট`); see [Typing Bangla](../input/typing-bangla.md). |
| `^` `:` `` ` `` after a letter | Part of the word: `^` makes a chandrabindu (`cha^d` gives `ছাঁদ`), `:` a visarga (`ba:` gives `বাঃ`), and `` ` `` keeps two letters apart (`` r`a `` gives `রআ`, not `রা`). |
| `Space` | Types the highlighted candidate (the first by default) and a space. |
| `Enter` | Types the highlighted candidate, then sends `Enter` to the application. |
| `1`–`9` | Types that candidate. |
| `↓` / `Tab` | Highlights the next candidate. |
| `↑` / `Shift+Tab` | Highlights the previous candidate. |
| `Backspace` | Removes the last letter. |
| `Esc` | Throws the word away. |
| Punctuation (`.` `,` `?` `!` and so on) | Types the highlighted candidate, then the mark. `.` becomes the Bangla full stop `।` and `$` becomes `৳`. |
| `Shift+Delete` | Forgets what was learned about the highlighted candidate. See [What it learns](../learning/learning.md). |
| `←` `→` `Home` `End` `Page Up` `Page Down` `Insert` `Delete` | Types the highlighted candidate, then the key does what it normally does in the application. |
| Any shortcut with `Ctrl`, `Alt` or `Super` | Types the highlighted candidate, then the shortcut goes to the application. |

Digits that are not choosing a candidate are typed as Bangla digits (`123` gives `১২৩`). To keep ASCII digits, set `bangla_digits = false`; see [The config file](../settings/config-file.md).

## After a space: next-word suggestions

| Key | What it does |
|---|---|
| `Tab` | Types the highlighted suggestion. |
| `↓` / `↑` | Moves the highlight. |
| `Shift+Delete` | Stops that suggestion from being offered again. |
| `Esc` | Hides the suggestions. |
| Any other key | Hides the suggestions and does its normal job. |

## Switching

| Key | What it does |
|---|---|
| `Ctrl+Alt+Space` | Switches between Bangla and English typing. A short notice (`বাংলা` or `English`) shows the new mode. |
| `Ctrl+Alt+M` | While a word or suggestions are shown: pins the window where you can drag it, or lets it follow the text cursor again. See [Moving the window](../settings/themes.md#moving-the-window). |
| Fcitx5's own switch key (often `Ctrl+Space`) | Switches between input methods, for example to your keyboard layout. |

## Fields Bornopata leaves alone

In password fields, and in fields an application marks as "disabled" for input methods, Bornopata does nothing and every key goes straight through.

## Changing shortcuts

The Bornopata shortcuts are in `~/.config/bornopata/config.toml`:

```toml
[shortcut]
toggle = "ctrl+alt+space"
forget = "shift+delete"
pin = "ctrl+alt+m"
```

A shortcut is one or more modifiers (`ctrl`, `alt`, `shift`, `super`) plus `space`, `delete`, a letter or a digit. A bare key is not allowed because it would break typing. The Server picks the change up by itself.
