---
title: Install on Windows
order: 2
description: Install the Bornopata preview on Windows 10 or 11, add it as a keyboard and type. It has worked on the one real machine it was tried on, so please send the log if something does not work for you.
---

Bornopata for Windows is an **early preview**: it builds, passes its automated tests and has worked on the one real Windows machine it was tried on, but few
machines have seen it. If something does not work, the log files tell what happened; see [If something does not work](#if-something-does-not-work).

## Install

1. Download `bornopata-<version>-windows-x86_64-setup.exe` and run it. Windows may say "Windows protected your PC" because the
   installer is not signed yet: choose **More info**, then **Run anyway**.
2. Accept the administrator prompt. The installer puts Bornopata into `C:\Program Files\Bornopata`, registers it as an input
   method and starts its background program (the Server), which also starts at every login.
3. **Sign out and sign in again**, or restart. Programs that were open during the installation do not see the new input method
   until they are restarted.
4. Open **Settings > Time & language > Language & region**, choose your language, then **Language options**, **Add a keyboard**
   and pick **Bornopata**. If Bangla (Bangladesh) is not in your list, add it first.

## Type

Switch to Bornopata with the language switcher in the taskbar (or `Win+Space`) in any text field and type `ami`. The letters show
as underlined Bangla, a window lists the candidates, and `Space` types the highlighted one. Typing, learning, emoji, phrases and
the keys work as on Linux: [Your first words](first-input.md) and [Keys and shortcuts](keys.md).

`Ctrl+Alt+Space` switches between Bangla and English inside Bornopata.

## What is different from Linux

- Candidates cannot be clicked yet; use the number keys or the arrows.
- There is no Bangla/English indicator in the taskbar yet.
- Your settings and what Bornopata learned are in `%APPDATA%\Bornopata`: `config.toml`, `phrases.tsv` and the learned data. Open
  **Bornopata Settings** from the Start menu to change settings.

## If something does not work

Bornopata writes two log files per day to `%LOCALAPPDATA%\Bornopata\logs`: `server.<date>.log` and `tsf.<date>.log`. They record
what happened, never what you typed. Send both when you report a problem, with the program you typed in and what you expected.

If nothing appears when you type, check that **Bornopata Server** is running in Task Manager; if it is not, start it from the
Start menu or run `bornopata-server.exe` in the installation folder.

## Uninstall

Settings > Apps > Installed apps > Bornopata > Uninstall. Your data in `%APPDATA%\Bornopata` is kept; delete the folder to remove
it too.
