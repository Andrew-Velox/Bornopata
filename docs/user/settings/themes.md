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
| `sakura` | ![sakura](images/theme-sakura.png) Bright pink, with a little mascot peeking over the corner of the window, a bow, a crown that bobs, and falling petals. Always light. |
| `shapla` | ![shapla](images/theme-shapla.png) Mint and teal with a water lily (shapla), the national flower of Bangladesh, and a girl with a long white braid who sits on the window's edge (lies along it in the horizontal layout). Always light. |
| `borsha` | ![borsha](images/theme-borsha.png) Rain blue with a cloud, and a girl in a yellow raincoat who peeks from behind the window with an umbrella (lies along its edge in the horizontal layout). Always light. |
| `jonaki` | ![jonaki](images/theme-jonaki.png) A night sky: deep blue with a moon and drifting fireflies, and a girl with a lantern of fireflies who leans on the window (lies along its edge in the horizontal layout). Always dark. |

You can also point `theme` at the path of a theme folder of your own.

`layout = "horizontal"` lists the candidates in a row instead of a column.

## Which window shows the candidates

- **Wayland applications** (Brave, Firefox, GTK and Qt programs): Bornopata's window appears at the text cursor, placed by your compositor, which also keeps it on screen.
- **X11 and XWayland applications** on Hyprland and other compositors with layer-shell: a small overlay window.
- **Everywhere else** (for example GNOME, or if anything fails): Fcitx5's own candidate panel, which shows the same list as plain text.

`mode` in `[panel]` forces one choice; the default `"auto"` picks the best available.

## Moving the window

Drag the window with the left mouse button and let go: it stays where you dropped it, and next time it shows there, in every application, until you free it. The place is kept in `window-position` in Bornopata's data folder (`~/.local/share/bornopata` on Linux, `%LOCALAPPDATA%\Bornopata` on Windows).

- **Windows and X11 applications:** just drag the window.
- **Wayland applications** (Brave, Firefox, GTK and Qt programs): your compositor places the popup there and gives programs no mouse for it, so it cannot be dragged. Press `Ctrl+Alt+M` while a word is shown: Bornopata's own window appears in the middle of your screen, a little below the center, and you drag it where you want it. From then on it shows there instead of at the text cursor.
- **Free it again:** right-click the window, or press `Ctrl+Alt+M` while it shows. It follows the text cursor from the next word on.

The window stays on the monitor it was dropped on (on Linux, dragging it to another monitor is not possible yet). The Fcitx5 panel, which is used when Bornopata's own window cannot be shown, cannot be moved.

## Limits

- Clicking a candidate with the mouse does not work in the popup window; use the keys. The window can be dragged, see above.
- The window needs a Bengali font, and a color emoji font for emoji. See [Troubleshooting](../help/troubleshooting.md).
