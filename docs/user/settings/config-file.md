---
title: The config file
order: 2
description: Every option in ~/.config/bornopata/config.toml with its default, and how to apply a change.
---

Settings live in `~/.config/bornopata/config.toml`. The Server writes the file with comments the first time it starts, and never overwrites it afterwards. **A saved change takes effect by itself**: the Server notices it within a second or two and starts over with the new settings, as long as nobody is in the middle of a word. If the file has a mistake, the Server keeps the old settings and writes the reason to its log. To restart it by hand:

```bash
systemctl --user restart bornopata
```

An option you leave out keeps its default, and an unknown option is ignored. If a value is wrong (a bad shortcut, say) the Server tells you which file and why when it starts.

## `[general]`

| Option | Default | What it does |
|---|---|---|
| `learning` | `true` | Remember the words you choose. See [What it learns](../learning/learning.md). |
| `bangla_digits` | `true` | Type digits as Bangla digits `০`–`৯`. `false` keeps ASCII digits. |
| `gloss` | `true` | Show the short English meaning beside candidates. See [English glosses](../learning/glosses.md). |
| `shortcuts` | `true` | `tarikh`, `somoy` and `bar` also offer the date, time and weekday. See [Date, time and weekday](../input/shortcuts.md). |
| `emoji` | `true` | Offer emoji next to the words that name them. See [Emoji](../input/emoji.md). |
| `autocorrect` | `true` | Type English words as written in their Bangla spelling: `facebook` gives ফেসবুক. See [English words](../input/autocorrect.md). |
| `language` | `"auto"` | The language of Bornopata Settings: `"auto"` follows the system language, `"en"` is English, `"bn"` is Bangla (বাংলা). The Server ignores it. |
| `log_level` | `"info"` | `"error"`, `"warn"`, `"info"` or `"debug"`. Logs never contain what you type. |

## `[shortcut]`

| Option | Default | What it does |
|---|---|---|
| `toggle` | `"ctrl+alt+space"` | Switch between Bangla and English typing. |
| `forget` | `"shift+delete"` | Forget the highlighted candidate or suggestion. |

A shortcut is one or more of `ctrl`, `alt`, `shift`, `super`, then `space`, `delete`, a letter or a digit, joined with `+`.

## `[panel]`

| Option | Default | What it does |
|---|---|---|
| `mode` | `"auto"` | Which window shows the candidates: `"auto"` (Bornopata's window where it can be shown, otherwise Fcitx5's panel), `"popup"` (only the input-method popup), `"native"` (only the overlay window), `"fcitx"` (always Fcitx5's panel). |
| `theme` | `"bornopata"` | `"bornopata"`, `"sakura"`, `"shapla"`, `"borsha"` or `"jonaki"`, or the path of a theme folder. |
| `dark` | `false` | The dark variant of themes that have one. |
| `layout` | `"vertical"` | `"vertical"` or `"horizontal"`. |

See [Themes and the window](themes.md).

## The whole file

```toml
[general]
learning = true
bangla_digits = true
gloss = true
shortcuts = true
emoji = true
autocorrect = true
language = "auto"
log_level = "info"

[shortcut]
toggle = "ctrl+alt+space"
forget = "shift+delete"

[panel]
mode = "auto"
theme = "bornopata"
dark = false
layout = "vertical"
```
