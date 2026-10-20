---
title: English mode
order: 6
description: Switch between Bangla and English with Ctrl+Alt+Space; in English mode every key goes straight to the application.
---

Press `Ctrl+Alt+Space` to switch to **English mode**. A short notice (`English`) appears. In English mode Bornopata is out of the way: every key goes straight to the application, so you type with your keyboard layout as if no input method were there. Press `Ctrl+Alt+Space` again to return to Bangla (`বাংলা`).

Each text field keeps its own mode, so switching in one window does not change another.

## Switching away completely

`Ctrl+Alt+Space` stays inside Bornopata. Fcitx5's own switch key (often `Ctrl+Space`) switches to another input method, for example a plain keyboard layout, and back.

## Change the shortcut

```toml
[shortcut]
toggle = "ctrl+alt+space"
```

in `~/.config/bornopata/config.toml`. The Server picks the change up by itself. See [Keys and shortcuts](../getting-started/keys.md).
