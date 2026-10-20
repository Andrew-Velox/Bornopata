# Bornopata Settings

The settings program is a native window drawn on `bamboo-slip` (the same engine as the candidate window), so the project has one UI stack and no GTK or Qt dependency. It is a shell: it reads and edits the config file and the user's data files, and contains no ranking, dictionary or conversion logic.

## Principles

- **The config file stays the single source of truth.** The program edits `~/.config/bornopata/config.toml` in place with `Config::set` (`toml_edit`, comments kept, atomic write). Hand edits and the program never fight over a private format.
- **Changes apply by themselves.** The Server checks the file's modification time on its once-a-second tick. A valid new file makes it exec itself (same pid), only when no session is composing; an invalid file is logged and ignored, so a half-typed edit never breaks typing.
- **Input beats everything else.** The program is a separate process. Closing it, crashing it or never installing it changes nothing about typing.
- **Destructive actions go through a marker file.** "Forget everything learned" writes `forget-learned` into the user data directory; the Server notices it on its tick, clears the learner and removes the marker. The program never opens the learner files.

## Pages

| Page | Edits |
|---|---|
| Typing | `[general]` learning, Bangla digits, gloss, shortcuts, emoji, log level |
| Keys | `[shortcut]` toggle and forget, recorded by pressing the keys |
| Appearance | `[panel]` mode, theme, dark, layout, with a live preview drawn by `borno-render` |
| Phrases | `phrases.tsv`: add, edit, remove; comments in the file are kept |
| Learning | counts of learned words, choices and pairs; forget everything |
| About | version, license, links (opened with `xdg-open`) |

## Parts

- `apps/settings` (`borno-settings`, binary `bornopata-settings`): Elm-style `App` (`view`, `update`) over a `Message` enum; one file per page in `view/pages/`.
- The widget kit (`SettingRow`, `Switch`, `Segmented`, `KeyRecorder`, `TextField`, dialogs, scroll area) and the winit host come from Qingjian and live in `bamboo-slip` behind the `host` feature.
- Launcher: `apps/settings/linux/bornopata-settings.desktop` and `bornopata.svg`, installed by `install.sh` and the release package; `StartupWMClass` matches the window's app id.

## Language

The program speaks English or Bangla. The English text stays at the call site and is the key: `t("Save")` returns the Bangla text from `src/i18n/bangla.rs` when `[general] language` is `bn` (or `auto` on a Bangla locale) and the English text when there is no entry. A test checks that every table entry is used by some source file. The Server ignores the setting. Terms people already read in English (Fcitx5, Wayland, Ctrl) and file names are left as they are. `--page NAME` opens the program on a page, which the screenshots for review use.

## Limits

- The previews are static frames, not the animated candidate window.
- Linux only for now; the host is cross-platform, the launcher entry and installer are not.
