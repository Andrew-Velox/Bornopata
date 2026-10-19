# Linux: Fcitx5 plugin and Server

How the pieces fit, how to build and install them on Arch with Hyprland, and what to check by hand.

## Parts

- **`borno-linux-server`** (`apps/linux/server`): holds one `Engine` per input context and all learned data, listens on a Unix socket (`$XDG_RUNTIME_DIR/bornopata.sock`, or `$BORNOPATA_SOCKET`). All input logic is in the `Router`.
- **`bornopata.so`** (`apps/linux/fcitx5`): a thin Fcitx5 addon. It forwards key and lifecycle events to the Server and draws the frame it gets back on Fcitx5's default panel. No custom rendering.
- Socket access is limited to the same user (`SO_PEERCRED`, mode 0600, private directory).

## Arch packages

```bash
sudo pacman -S fcitx5 fcitx5-gtk fcitx5-qt fcitx5-configtool nlohmann-json pkgconf cmake noto-fonts
```

`fcitx5` already ships the development headers on Arch. Rust comes from `rustup` (the toolchain in `rust-toolchain.toml`). `noto-fonts` provides a Bangla font (Noto Sans Bengali).
Debian and Ubuntu: `cmake g++ pkg-config python3 libfcitx5core-dev libfcitx5utils-dev libfcitx5config-dev nlohmann-json3-dev`. FreeBSD: `pkg install bash cmake pkgconf python3 fcitx5 nlohmann-json rustup-init`.

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
| Tab on next-word suggestions, or a click | take the suggestion |
| Left, Right, Home, End, Delete ... | commit the word, then move |
| Ctrl / Alt / Super chords | commit the word, then the application gets the chord |
| `ctrl+alt+space` (configurable) | switch between Bangla and English; English passes every key through |

## Configuration

`~/.config/bornopata/config.toml` is written with comments on first start. Restart the Server after editing (`systemctl --user restart bornopata`).

```toml
[general]
learning = true          # remember choices (kept in ~/.local/share/bornopata)
bangla_digits = true
log_level = "info"       # logs in ~/.local/state/bornopata/logs

[shortcut]
toggle = "ctrl+alt+space"
```

Learned data (`user-words.tsv`, `user-choices.tsv`, `user-ngram.tsv`) is written a few seconds after the last commit and when the Server stops. An optional language model goes in `~/.local/share/bornopata/lm.bnd`. Typed text is never written to the logs.

## Protocol

Framing: a 4-byte little-endian length, then a JSON body (`borno_platform::protocol`). Version `PROTOCOL_VERSION = 1`; the Server refuses any other and there is no compatibility layer, the Server and the plugin are installed together. Bump the number whenever a message changes meaning and change the plugin in the same commit.

- Plugin to Server: `Open{session, app, protocol, generation, context}`, `Close{session}`, `Event{session, event}` with events `Capabilities`, `Reset`, `Key{event, release}`, `Deactivate`, `Focus`, `Candidate{identity, index}`. The first event of a session must be `Capabilities`.
- Server to plugin: `Opened`, `Result{session, outcome, commit, identity, frame}`, `Ignored`. `commit` is applied before the frame is drawn. `identity.revision` grows with every reply so a stale reply or a click on a replaced frame is dropped.
- `Frame`: `roman`, `preedit` (Bangla text of the highlighted candidate), `candidates` (at most nine, each with `text` and `source`), `highlight`, `suggesting`, `mode`.

## Tests

```bash
cargo test                                                       # includes the Router and socket tests
cmake -S apps/linux/fcitx5 -B target/fcitx5 -DBUILD_TESTING=ON && cmake --build target/fcitx5
cargo build -p borno-linux-server                                # the real-Server tests need the binary
env -u DBUS_SESSION_BUS_ADDRESS ctest --test-dir target/fcitx5 --output-on-failure
```

The C++ tests cover the panel against a mock Server, the framework event order, transport failures, and typing, privacy and reentrancy against the real Server. The `multiplex` and `reentrant` suites of the pinyin plugin were dropped with the features they tested.
