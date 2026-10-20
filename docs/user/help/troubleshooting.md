---
title: Troubleshooting
order: 2
description: Fixes for Bornopata that does not appear in Fcitx5, types Latin letters, shows empty boxes, has no candidate window, or does not learn.
---

## It types Latin letters, not Bangla

The plugin passes your keys straight through when it cannot reach the Server. Check the Server:

```bash
systemctl --user status bornopata
```

If it is not running, start it with `systemctl --user enable --now bornopata`. If it fails to start, `journalctl --user -u bornopata` and the log in `~/.local/state/bornopata/logs/` say why (a wrong value in `config.toml` is named there).

Also check that you really are in Bangla mode, not English mode: press `Ctrl+Alt+Space` and look for the `বাংলা` notice. See [English mode](../input/english-mode.md).

## Bornopata is not in the Fcitx5 list

- Restart Fcitx5 (`fcitx5 -r`) after installing.
- In the Fcitx5 configuration tool, untick **Only show current language**, or search for "Bornopata".
- Make sure the files are there: `~/.local/share/fcitx5/addon/bornopata.conf` and `~/.local/lib/fcitx5/bornopata.so`. If Fcitx5 refuses to load the plugin, it was probably built against another Fcitx5 version; install from source. Running `fcitx5 -r` in a terminal, or `fcitx5-diagnose`, prints why.

## Bangla shows as empty boxes

Install a Bengali font, for example Noto Sans Bengali (`noto-fonts` on Arch, `fonts-noto-core` on Debian and Ubuntu), and restart the application.

## There is no candidate window, or it is not where the cursor is

- In Wayland applications the window is placed by your compositor. The application must use the Wayland input method (a Chromium-based browser running under XWayland uses the overlay or Fcitx5's panel instead).
- Without a Bengali font the window cannot be drawn and Fcitx5's own panel is used.
- Force a choice with `mode` in `[panel]` of [the config file](../settings/config-file.md): `"fcitx"` always uses Fcitx5's panel, which works everywhere.
- Clicking a candidate does not work; choose with the keys.

## Emoji show as boxes or do not appear

Install a color emoji font (Noto Color Emoji) for the candidate window, and make sure `emoji = true`. Emoji also appear only when a word names them: try `hridoy` or `heart`.

## It does not learn, or learns the wrong word

- Password fields and fields marked sensitive are never learned; neither are phrases, dates and times.
- `learning = false` in the config turns it off.
- Learned data is saved a few seconds after you stop typing.
- A wrong word that keeps coming first: highlight it and press `Shift+Delete`.

## The date or time is wrong

`tarikh`, `somoy` and `bar` use your computer's clock and time zone. Check `timedatectl`.

## A change to the config has no effect

The Server notices a saved change within a second or two and starts over with it, as long as nobody is in the middle of a word. If the file has a mistake (a bad value, a missing bracket) it keeps the old settings and writes the reason to its log, so check the log; the next valid save takes effect. To restart by hand: `systemctl --user restart bornopata`.

## Still stuck

See [Feedback](feedback.md) and include the version (`borno-linux-server --version`), your desktop, the application, and the relevant lines of the log.
