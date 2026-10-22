---
title: Install on Windows
order: 2
description: Install the Bornopata preview on Windows 10 or 11, add it as a keyboard and type. It has worked on the one real machine it was tried on, so please send the log if something does not work for you.
---

Bornopata for Windows is an **early preview**: it builds, passes its automated tests and has worked on the one real Windows machine it was tried on, but few
machines have seen it. If something does not work, the log files tell what happened; see [If something does not work](#if-something-does-not-work).

## Install

1. Download `bornopata-<version>-windows-x86_64-setup.exe` and run it. If Windows says "Windows protected your PC" because the
   installer is not signed yet, choose **More info**, then **Run anyway**.
2. Accept the administrator prompt and click through the installer. It puts Bornopata into `C:\Program Files\Bornopata`,
   registers it as an input method, adds the **Bornopata** keyboard to your languages (under Bangla (Bangladesh)) and starts its
   background program (the Server), which also starts at every login.
3. Programs that were open during the installation do not offer the new keyboard until they are closed and opened again; signing
   out and in again does the same for all of them.

If Bornopata is not in the language list afterwards, add it by hand: **Settings > Time & language > Language & region**, your
language (add Bangla (Bangladesh) first if it is missing), **Language options**, **Add a keyboard**, **Bornopata**. What the
installer did about it is written in `%LOCALAPPDATA%\Bornopata\logs\add-keyboard.log`.

## Type

Switch to Bornopata with `Win+Space` (or the language switcher in the taskbar) in any text field and type `ami`. The letters show
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
