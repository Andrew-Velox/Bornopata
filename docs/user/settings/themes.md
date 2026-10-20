---
title: Themes and the window
order: 3
description: The five candidate-window themes (bornopata, sakura, shapla, borsha, jonaki), light and dark, vertical or horizontal, and which window is used in which application.
---

## Themes

Set `theme` in `[panel]` of [the config file](config-file.md); the window changes within a second or two.

| Theme | Look |
|---|---|
| `bornopata` (default) | ![bornopata](images/theme-bornopata.png) Soft green and white. Has a dark variant (`dark = true`). |
| `sakura` | ![sakura](images/theme-sakura.png) Pink with a two-tone border and a cherry blossom in the corner. Always light. |
| `shapla` | ![shapla](images/theme-shapla.png) Mint and teal with a water lily (shapla), the national flower of Bangladesh. Always light. |
| `borsha` | ![borsha](images/theme-borsha.png) Rain blue with a cloud. Always light. |
| `jonaki` | ![jonaki](images/theme-jonaki.png) A night sky: deep blue with a moon and drifting fireflies (`jonaki`). Always dark. |

You can also point `theme` at the path of a theme folder of your own.

`layout = "horizontal"` lists the candidates in a row instead of a column.

## Which window shows the candidates

- **Wayland applications** (Brave, Firefox, GTK and Qt programs): Bornopata's window appears at the text cursor, placed by your compositor, which also keeps it on screen.
- **X11 and XWayland applications** on Hyprland and other compositors with layer-shell: a small overlay window.
- **Everywhere else** (for example GNOME, or if anything fails): Fcitx5's own candidate panel, which shows the same list as plain text.

`mode` in `[panel]` forces one choice; the default `"auto"` picks the best available.

## Limits

- Clicking a candidate with the mouse does not work in the popup window; use the keys.
- The window needs a Bengali font, and a color emoji font for emoji. See [Troubleshooting](../help/troubleshooting.md).
