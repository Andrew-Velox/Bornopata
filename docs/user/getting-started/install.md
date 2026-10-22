---
title: Install
order: 1
description: Install Bornopata on Linux from the AUR, the release tarball or source; one command starts the server and adds it to Fcitx5.
---

Bornopata has two parts: a small **Server** that does the work in the background, and a **plugin** for Fcitx5, the input method framework, that sends your keys to it. Both install into your home directory, so no root access is needed.

## What you need

- Linux with **Fcitx5** (version 5 or later) running. On Arch: `sudo pacman -S fcitx5 fcitx5-gtk fcitx5-qt fcitx5-configtool`.
- A **Bengali font**, for example Noto Sans Bengali (`noto-fonts` on Arch, `fonts-noto-core` on Debian and Ubuntu). Without one, Bangla shows as empty boxes.
- For emoji in the candidate window, a color emoji font such as Noto Color Emoji.
- `systemd` (user services) to keep the Server running. Other setups can start `~/.local/bin/borno-linux-server` themselves.

The candidate window appears at your text cursor in Wayland applications through Fcitx5's Wayland support. In other applications it uses Fcitx5's own candidate panel or a small overlay window; see [Themes and the window](../settings/themes.md).

## Install on Arch Linux (AUR)

```bash
yay -S bornopata
bornopata-setup
```

`bornopata-setup` is run once, as your own user: see [What the setup does](#what-the-setup-does). Updating is `yay -S bornopata` and `bornopata-setup` again.

## Install from the release

1. Download `bornopata-<version>-linux-x86_64.tar.gz` from the releases page and unpack it.
2. In the unpacked folder run `./install.sh`.

`install.sh` puts everything under `~/.local` (use `--prefix DIR` for another place), never overwrites files it did not install itself, and then runs the setup below. The prebuilt plugin is built against one Fcitx5 version. If Fcitx5 cannot load it, build from source instead.

## What the setup does

`bornopata-setup` (run for you by `install.sh`) does the three things that used to be separate steps:

1. Starts the Server as a systemd user service, and enables it so it starts at every login.
2. Adds **Bornopata** to your Fcitx5 input methods. It keeps a copy of your list as `~/.config/fcitx5/profile.bornopata-backup` and touches nothing else in it.
3. Restarts Fcitx5, so it loads the plugin.

Fcitx5 restarts for a moment, so save your work first. If Fcitx5 has never been set up (no `~/.config/fcitx5/profile` yet), open `fcitx5-configtool` once and run `bornopata-setup` again. Pass `--no-service` or `--no-fcitx` to skip a part.

## Install from source

You need Rust (from `rustup`), `cmake`, a C++ compiler, `pkg-config`, `python3`, the Fcitx5 development files and `nlohmann-json`. Then:

```bash
git clone <repository>
cd bornopata
apps/linux/scripts/install.sh
```

This also runs the setup above. This installer does not include the dictionary: without `~/.local/share/bornopata/words.bnd` only a small built-in word list answers, so download `words.bnd` from the dictionary data release (named `data-v1` or newer) on the releases page and put it there. On Debian and Ubuntu the packages are `cmake g++ pkg-config python3 libfcitx5core-dev libfcitx5utils-dev libfcitx5config-dev nlohmann-json3-dev libwayland-dev libwayland-bin wayland-protocols`.

## Check that it works

Open any text field, switch to Bornopata (Fcitx5's usual switch key, often `Ctrl+Space`) and type `ami`. A candidate window should show `আমি`, and `Space` types it. If not, see [Troubleshooting](../help/troubleshooting.md).

Update by installing again over the old version. Remove it with `uninstall.sh`; see [Uninstall](../help/uninstall.md).
