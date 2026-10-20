# Linux: Fcitx5 plugin and Server

How the pieces fit, how to build and install them on Arch with Hyprland, and what to check by hand.

## Parts

- **`borno-linux-server`** (`apps/linux/server`): holds one `Engine` per input context and all learned data, listens on a Unix socket (`$XDG_RUNTIME_DIR/bornopata.sock`, or `$BORNOPATA_SOCKET`). All input logic is in the `Router`.
- **`bornopata.so`** (`apps/linux/fcitx5`): a thin Fcitx5 addon. It forwards key and lifecycle events and the text cursor position to the Server and draws the frame it gets back: on Fcitx5's default panel, or, when the Server says the frame is `native`, only the inline preedit. For Wayland input contexts it also shows the bitmaps the Server draws in the input method's popup surface (`src/popup/`).
- **The self-drawn window** with the Bornopata themes, in two forms; see [../design/candidate-window.md](../design/candidate-window.md) and [../design/popup-surface.md](../design/popup-surface.md):
  - the **input-method popup** (`apps/linux/server/src/popup`, shown by the plugin): for Wayland-native clients (Brave, Firefox, GTK, Qt). The compositor places it at the caret and flips it at screen edges. Used when the plugin can create the popup surface (Fcitx5 5.1 with the `waylandim` frontend);
  - the **layer-shell overlay** (`crates/borno-panel`, started by the Server): for X11 and XWayland clients with a known cursor rectangle on Hyprland and other wlroots compositors.
  Everything else (GNOME, KDE, any failure) keeps Fcitx5's panel.
- Socket access is limited to the same user (`SO_PEERCRED`, mode 0600, private directory).

## Arch packages

```bash
sudo pacman -S fcitx5 fcitx5-gtk fcitx5-qt fcitx5-configtool nlohmann-json pkgconf cmake noto-fonts wayland wayland-protocols
```

`fcitx5` already ships the development headers on Arch. `wayland` (the client library and `wayland-scanner`) and `wayland-protocols` are needed for the popup surface; without any of them, or without the Fcitx5 Wayland module, CMake builds the plugin without it (`-DBORNOPATA_WAYLAND_POPUP=OFF` does the same) and Wayland clients get Fcitx5's panel. Rust comes from `rustup` (the toolchain in `rust-toolchain.toml`). `noto-fonts` provides a Bangla font (Noto Sans Bengali).
Debian and Ubuntu: `cmake g++ pkg-config python3 libfcitx5core-dev libfcitx5utils-dev libfcitx5config-dev nlohmann-json3-dev` (and `libwayland-dev libwayland-bin wayland-protocols` for the popup surface; `libsystemd-dev` to run the plugin tests). FreeBSD: `pkg install bash cmake pkgconf python3 fcitx5 nlohmann-json rustup-init`.

## Build and install (user directory, no root)

```bash
apps/linux/scripts/install.sh            # release build, installs under ~/.local
apps/linux/scripts/install.sh --debug    # debug build
apps/linux/scripts/install.sh --prefix /some/absolute/dir
apps/linux/scripts/uninstall.sh          # removes only what the install manifest lists
```

It installs `bin/borno-linux-server`, `lib/fcitx5/bornopata.so`, the Fcitx5 addon and input method files under `$XDG_DATA_HOME/fcitx5`, and a systemd user unit `~/.config/systemd/user/bornopata.service`.
Configuration, learned data and logs are never touched by install or uninstall.

## Enable on Hyprland

1. Start Fcitx5 with the session, and the Server. In `~/.config/hypr/hyprland.conf`:

   ```
   exec-once = fcitx5 -d
   env = XMODIFIERS,@im=fcitx
   env = QT_IM_MODULE,fcitx
   ```

   Do not set `GTK_IM_MODULE`: GTK3 and GTK4 apps talk to Fcitx5 through the Wayland text-input protocol. Chromium and Electron apps need `--enable-wayland-ime` (and `--wayland-text-input-version=3` on older builds).
2. Start the Server: `systemctl --user daemon-reload && systemctl --user enable --now bornopata`. Without systemd, add `exec-once = ~/.local/bin/borno-linux-server` instead.
3. Restart Fcitx5 (`fcitx5 -r`), open `fcitx5-configtool`, untick "Only show current language", add **Bornopata** to the input method list.
4. Select it with Fcitx5's own switch (default Ctrl+Space), then type.

## Keys

| Key | Effect while typing Bangla |
|---|---|
| letters, `` ` `` `^` `:` after a letter | build the Roman word; case matters (`t` ত, `T` ট) |
| Space | commit the word and a space; suggests next words if it knows any |
| Enter | commit the word, then the application gets Enter |
| `.` `,` `?` and other punctuation | commit the word, then the converted mark (`.` gives ।) |
| digits | Bangla digits (০-৯); `bangla_digits = false` keeps ASCII. While a word is shown, 1-9 pick that candidate |
| Backspace | edit the Roman word; passes through when it is empty |
| Esc | cancel the word |
| Up / Down / Tab (Shift+Tab back) | move the highlight |
| Up / Down on next-word suggestions | move through them (they wrap); the arrow does not reach the application |
| Tab on next-word suggestions, or a click | take the highlighted suggestion |
| Esc, or any other key, on next-word suggestions | dismiss them |
| Left, Right, Home, End, Delete ... | commit the word, then move |
| Ctrl / Alt / Super chords | commit the word, then the application gets the chord |
| `ctrl+alt+space` (configurable) | switch between Bangla and English; English passes every key through |

## The dictionary

Without a dictionary only the `upodesh` scaffold list answers and nothing is completed. Build `words.bnd` once (see [assets/dictionary/README.md](../../assets/dictionary/README.md): about two minutes from the Wikipedia dump and the Common Voice sentences) and put it where the Server looks:

```bash
install -Dm644 words.bnd ~/.local/share/bornopata/words.bnd     # or point $BORNOPATA_DICT at it
systemctl --user restart bornopata
```

Then typing `bangl` offers বাংল first (what you typed) followed by বাংলাদেশ, বাংলা ... (completions, most frequent first). Pick one with its number or the arrow keys; Space still commits what you typed. The Server log says which dictionary it opened.

## Configuration

`~/.config/bornopata/config.toml` is written with comments on first start. Restart the Server after editing (`systemctl --user restart bornopata`).

```toml
[general]
learning = true          # remember choices (kept in ~/.local/share/bornopata)
bangla_digits = true
gloss = true             # English gloss with part of speech beside each candidate ("n. rice"); needs gloss-en.bnd
log_level = "info"       # logs in ~/.local/state/bornopata/logs

[shortcut]
toggle = "ctrl+alt+space"

[panel]
mode = "auto"            # auto: popup for Wayland clients, layer-shell window for X11, else Fcitx5's panel;
                         # popup: only the input-method popup; native: only the layer-shell window; fcitx: always Fcitx5's panel
theme = "bornopata"      # or "sakura", "shapla", "borsha", "jonaki", or the path of a theme directory
dark = false             # dark variant of themes that have one
layout = "vertical"      # or "horizontal"
```

Fonts: the window needs a Bengali font (Noto Sans Bengali from `noto-fonts` on Arch); `journalctl --user -u bornopata` or the log says which panel was chosen.

The gloss file `gloss-en.bnd` (installed next to `words.bnd` by `install.sh`, or `$BORNOPATA_GLOSS`) is mapped at Server start when `gloss = true`; the log says how many entries were loaded. See [../design/glosses.md](../design/glosses.md).

Your own phrases go in `~/.local/share/bornopata/phrases.tsv` (the Server writes a commented template on first start and never overwrites it). One phrase per line, `code<TAB>text`: a code is Roman letters only and is matched exactly as typed, `\n` `\t` `\\` in the text give a line break, a tab and a backslash, and several lines with one code give several candidates in file order. Typing the code puts the text first in the list (marked like a learned choice); the ordinary words stay behind it. Phrases are never learned, so private text lives only in that file. The Server reads the file again within a couple of seconds of a save; a bad line is skipped and logged as a warning with its line number.

Learned data (`user-words.tsv`, `user-choices.tsv`, `user-ngram.tsv`) is written a few seconds after the last commit and when the Server stops. An optional language model goes in `~/.local/share/bornopata/lm.bnd`. Typed text is never written to the logs.

## Protocol

Framing: a 4-byte little-endian length, then a JSON body (`borno_platform::protocol`). Version `PROTOCOL_VERSION = 4`; the Server refuses any other and there is no compatibility layer, the Server and the plugin are installed together. Bump the number whenever a message changes meaning and change the plugin in the same commit.

- Plugin to Server: `Open{session, app, protocol, generation, context, popup}` (`popup` is the token of the popup channel when the context can show the popup, else null), `Close{session}`, `Event{session, event}` with events `Capabilities`, `Reset`, `Key{event, release}`, `Deactivate`, `Focus`, `Candidate{identity, index}` and `Cursor{x, y, width, height, surface_relative}` (the text cursor, sent when it changes and before a key). The first event of a session must be `Capabilities`.
- Server to plugin: `Opened`, `Result{session, outcome, commit, identity, frame}`, `Ignored`. `commit` is applied before the frame is drawn. `identity.revision` grows with every reply so a stale reply or a click on a replaced frame is dropped.
- **Popup channel**: a second connection starts with `Attach{protocol, token}` (answered `Attached`) and then carries only popup messages. Server to plugin: `Pool{pool, size}` with a `memfd` as `SCM_RIGHTS` ancillary data, `Present{context, pool, buffer, offset, width, height, stride, scale_120, logical_width, logical_height}` (premultiplied ARGB8888, cropped to the card) and `Hide`. Plugin to Server: `Released{buffer}` and `Scale{scale_120}`. The plugin reads it from Fcitx5's event loop without blocking; the Server writes from its render thread, so typing never waits for a render. Details: [../design/popup-surface.md](../design/popup-surface.md).
- `Frame`: `roman`, `preedit` (Bangla text of the highlighted candidate), `candidates` (at most nine, each with `text`, `source`: `Learned`, `Direct`, `Dictionary`, `Completion` or `Next`, and an optional `gloss {pos, text}`), `highlight`, `suggesting`, `mode`, `native` (the Server shows the frame in its own window, so the plugin draws no candidate list).

## Tests

```bash
cargo test                                                       # includes the Router and socket tests
cmake -S apps/linux/fcitx5 -B target/fcitx5 -DBUILD_TESTING=ON && cmake --build target/fcitx5
cargo build -p borno-linux-server                                # the real-Server tests need the binary
env -u DBUS_SESSION_BUS_ADDRESS ctest --test-dir target/fcitx5 --output-on-failure
```

The C++ tests cover the panel against a mock Server, the framework event order, transport failures, and typing, privacy and reentrancy against the real Server. The popup tests (`popup-*`) run the channel against the mock Server with a fake presenter; `real-popup` runs the real Server's render thread; `proxy-finder` checks the search for the raw proxy in Fcitx5's wrapper object. The `multiplex` and `reentrant` suites of the pinyin plugin were dropped with the features they tested.
