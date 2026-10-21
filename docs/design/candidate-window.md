# The candidate window

Decided and built 2026-10-20 for Linux (Wayland first). A self-drawn window replaces Fcitx5's panel where the compositor allows it, so Bornopata can show
what the default panel cannot: where a word came from, next-word hints, soft shadows, themes and animation.

## Parts

- `bamboo-slip` (from Qingjian, GPL side of its dual license): scene tree, flexbox layout (`taffy`), `tiny-skia` rasterizing, `cosmic-text` shaping, SVG, animation. Pixels in, a premultiplied RGBA bitmap out.
- `borno-render`: `Frame` + `Layout` + `Theme` + scale gives a bitmap, the click regions and the time of the next animation frame. A frame is a display shape: the typed Roman letters, rows marked by where each word came from (typed, learned, completion, next-word), a highlight, a hint chip and the typing mode. Themes are JSON (`theme.json`, comments allowed) with colour variables, components, gradients, shadows, SVG images, transitions and looping animations; the format is described in the upstream design notes (`git fetch upstream`, `docs/design/theme.md`) and unchanged except for the data names listed below.
- `borno-panel`: a thread that owns the Wayland connection and shows the bitmap on a wlr-layer-shell surface; the popup placement; the compositor geometry.
- The Server's `Router` hands each reply's frame to a `PanelSink`; the Fcitx5 plugin reports the text cursor and draws nothing in the panel when the frame says `native`.

## Why a layer-shell window positioned by compositor IPC

A Wayland client knows its text cursor only relative to its own surface. The `input-method-v2` popup surface would be placed by the compositor exactly, but only the process that owns the input-method object (Fcitx5's Wayland frontend) can create it. The plugin can reach that object after all, so Wayland-native clients now use it ([popup-surface.md](popup-surface.md)); the layer-shell window stays for X11 and XWayland clients, whose cursor rectangle is known. Alternatives and why they lost:

| Option | Verdict |
|---|---|
| Server draws a layer-shell overlay, positioned from the cursor rectangle plus the window position from the compositor's IPC | **Chosen.** Pure Rust, full control of the bitmap, testable on a real desktop |
| Fcitx5 UI addon using the Wayland input popup | private Fcitx5 interface, replaces the panel for every input method, C++ drawing |
| kimpanel over D-Bus | loses the rich frame (sources, next words); same coordinate problem |
| Bornopata as its own `input-method-v2` client without Fcitx5 | loses the Fcitx5 ecosystem and XWayland; a rewrite |

Cost of the choice: the popup is placed at window origin + cursor rectangle, so a client that draws a shadow margin inside its surface (some GTK apps) is a few tens of pixels off. XWayland clients report global coordinates and are exact. GNOME has no layer-shell and keeps the default panel; KDE has layer-shell but no simple window-geometry IPC yet, so it keeps the default panel too. Hyprland works today; Sway is next (`swaymsg -t get_tree`).

## Behavior

- `[panel] mode = "auto"` starts the window at Server launch and falls back to Fcitx5's panel, with a log line, when there is no Wayland session or no layer-shell. `fcitx` never starts it. If the window dies later, the Router drops it and the plugin draws again from the next reply on.
- A frame is shown only once the plugin has reported a cursor; before that the plugin draws.
- An all-zero rectangle does not count as a cursor. Fcitx5 on Hyprland hands that over for Wayland-native clients such as Chromium, because the compositor only gives the caret position to the input method's own popup surface; a window placed by it would sit in the corner of the app. Those frames are drawn in the compositor-placed popup surface when the plugin can create it ([popup-surface.md](popup-surface.md)), and by Fcitx5's panel otherwise.
- The surface is on the overlay layer, anchored top-left with margins that place it, takes no keyboard focus, has an empty input region (clicks go to the application) and is destroyed when the frame is empty.
- Placement: below the cursor by 6 px, flipped above it when there is no room below, pushed in from the monitor's edges. The monitor comes from Hyprland's `monitors`; the surface is created on that output.
- Scale: rendered at the monitor's scale rounded up to an integer, with `set_buffer_scale`. Fractional scales (`wp_fractional_scale` with `wp_viewporter`) are not done yet, so a 1.25 monitor shows a slightly soft window.
- Mouse clicks on rows are not handled yet; keys and Tab already select.

## Text

`bamboo-slip` shapes with `Shaping::Advanced` (rustybuzz), which handles Bengali conjuncts and vowel-sign reordering. What it needs is a font: the Linux list now finds Noto Sans Bengali and otherwise any font file with a Bengali name under `/usr/share/fonts`. Without one every glyph is a box. The themes put `Noto Sans Bengali` first in the family chain.

## Themes

The horizontal list draws each candidate as a small column: the number and the word on top and the English meaning on a line below, with the highlight covering both lines. The line is reserved only when some candidate has a meaning (`when: "annotations"`). All built-in themes share this item. Mascots stand or lie outside the card and each picture is cut, sized and described in its theme's `README.md`. There is no paging: the Server shows at most nine candidates, so there are no page arrows.

- `bornopata`: white-to-mint card, radius 14, layered shadow, green highlight pill with a gradient, star for learned words, trailing ellipsis for completions, arrow for next-word suggestions, a `Tab` chip.
- `sakura`: bright pink glass with a mascot (a chibi girl with a green leaf hairpin, `images/character.png`, supplied by the project owner; see the theme's README) resting her paws on the top-left edge of the card, a bow, an outline crown that bobs, petals drifting and blossoms rocking in a loop (the animation is why the window asks for repeated frames). `meta.popup_overhang` says how far this decoration may hang over the card in the Wayland popup. Locked to light. Built-in themes list their images in `src/theme/mod.rs`; a test fails when a file is not listed.
- `shapla`: a cool water-lily card (the national flower of Bangladesh), white and pale teal, a lily in the top-right corner and two ripples spreading at the bottom. The mascot is a chibi girl with a long white-silver braid fading to teal and a water lily in her hair: sitting on the top edge near the right corner in the vertical layout, lying along it with the braid trailing in the horizontal one. Locked to light.
- `borsha`: a rainy-day card, blue-grey with squarer corners, a cloud in the corner and raindrops falling through the window. The mascot is a girl in a yellow raincoat: peeking from behind the card's right edge with a blue umbrella in the vertical layout, lying along the top edge beside a closed umbrella in the horizontal one. Locked to light.
- `jonaki`: a night card, deep indigo with a gold border, a crescent moon in the corner and three fireflies drifting and glowing. The mascot is a girl with long indigo hair and a lantern jar of fireflies: leaning on the card's right edge in the vertical layout, lying along the top edge in the horizontal one. Locked to dark.
- Their decoration sits inside the card (the top line leaves room for it), because the input-method popup shows the card cropped to its rectangle and would cut off anything hanging over the corner.
- All pictures are SVGs drawn for Bornopata (`themes/*/images`) under GPL-3.0-or-later. Nothing from Qingjian's themes, whose artwork has its own copyrights, is included.

## Frame changes from Qingjian

`Row.cloud` and `Row.code` became `Row.source`; conditions `learned`, `completion`, `next`, `hint`, `suggesting` replace `cloud`, `code`, `trailing`, `page`; `Frame.sentence`, `status`, `footer`, the grid layout and the status bar are gone; `Mode` is only `english`; the `cloud` icon became `star`.

## Open

Fractional scale, clicking rows, Sway and KDE placement, hover states, a theme chooser in a settings UI.
