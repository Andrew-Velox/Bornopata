# Changelog

Written by hand by the maintainers at release time; pull requests do not edit it.

## 0.1.0-alpha.3 - 2026-10-22

- Installing is shorter. On Linux `install.sh` also starts the Server, adds Bornopata to your Fcitx5 input methods and restarts Fcitx5; after `yay -S bornopata` one command, `bornopata-setup`, does the same, and `uninstall.sh` takes it all out again. On Windows the installer adds the Bornopata keyboard to your language list itself, so after it you only press `Win+Space` (that part has not been tried on a real machine yet).
- The candidate window can be moved: drag it with the mouse and it shows there from then on, in every application. In Wayland applications press `Ctrl+Alt+M` while a word is shown to get a window you can drag; right-click it or press the shortcut again to let it follow the text cursor. The shortcut is `pin` under `[shortcut]`. Dragging works on Windows too, but is untested there.
- The Windows installer from `0.1.0-alpha.2` has now been run on a real Windows machine and everything worked. It is still unsigned, so Windows may ask for confirmation (More info, then Run anyway), and few machines have seen it.

## 0.1.0-alpha.2 - 2026-10-21

- English words typed as written come out in Bangla: `facebook` gives ফেসবুক, `computer` কম্পিউটার, and an ending can follow (`facebooke` gives ফেসবুকে, `tvte` টিভিতে). About 360 words, built into the program; the setting is `autocorrect` under `[general]`, also on the Typing page of Bornopata Settings.
- A Windows version, as an early preview: a text service for every application (`bornopata_tsf.dll`), the Server with the candidate window (`bornopata-server.exe`) and an installer for Windows 10 and 11. It builds and its automated tests pass, but nobody has run it on a real Windows machine yet, so expect problems. The installer is not signed, so Windows asks for confirmation (More info, then Run anyway). If something fails, the logs are in `%LOCALAPPDATA%\Bornopata\logs` and never contain what you type; please open an issue with them.
- The Sakura theme has a pink mascot peeking over the corner of the window, with a bow, a crown that bobs and falling petals. The Wayland popup now shows decoration that hangs over the card, up to what the theme declares.
- The Shapla, Borsha and Jonaki themes have mascots too: beside or on the vertical window, and lying along the top edge of the horizontal one. Every mascot wears modest long clothes.
- The horizontal candidate list shows the English meaning under each word, like the vertical list does beside it.
- A new logo: a white ব with two leaves on green, in the settings program, the launcher entry, Fcitx5's list of input methods and the website.
- Words the letters cannot spell are found too: `moddhe` gives মধ্যে, `porjonto` পর্যন্ত, `dukkho` দুঃখ.
- Bornopata Settings speaks Bangla: set `language = "bn"` in the config file or pick বাংলা on the Typing page; the default follows the system language.
- Bengali conjuncts such as প্র, ক্রি, ন্ত্র and স্ব are drawn correctly in the candidate window and the settings program (they showed a stray hasanta and a full-width letter).
- A built-in word pair model (2 MB, from the Common Voice Bengali sentences): next-word suggestions from the first day, and the previous word helps choose between spellings. `Shift+Delete` hides a suggestion for good.
- Bornopata Settings: a settings program in the application launcher (typing, keys, appearance with a live preview, phrases, learning, about).
- The Server starts over by itself within a second or two of a saved config change, and forgets everything learned when the settings program asks.

## 0.1.0-alpha.1 - 2026-10-20

First preview, for Linux with Fcitx5 only. Tested on Arch with Hyprland and Brave; other desktops are untested.

Install: unpack the tarball, run `./install.sh`, then `systemctl --user enable --now bornopata`, restart Fcitx5 and add "Bornopata" in its configuration tool. The prebuilt plugin is built against Ubuntu's Fcitx5; if Fcitx5 cannot load it, build from the source archive (`docs/notes/linux-fcitx5.md`).

Known limits: no mouse clicks on the candidate window; the English glosses cover only part of the 5,000 most frequent words and are not reviewed by a Bangla speaker; ranking is tuned on a small hand-written set.

What is in it:

- Project started as Bornopata, imported from the Qingjian pinyin input method (history up to tag `qingjian-import-base`).
- Stripped to Core, CLI and the Linux Server with the Fcitx5 shell.
- Real word data: `tools/dict-build` builds a 413k-word dictionary from Bengali Wikipedia and Common Voice counts; typing shows frequency-ranked spellings and prefix completions (`bangl` gives বাংলা, বাংলাদেশ ...).
- Crates renamed to `borno-*`; the data container extension is now `.bnd` and the model extension `.bnm`.
- Wayland-native applications (Brave, Firefox, GTK, Qt) show the candidate window at the text cursor through the input-method popup surface.
- English glosses beside the candidates (`n. rice`), from Wiktionary (CC BY-SA 4.0), for part of the 5,000 most frequent words.
- Five candidate-window themes: `bornopata`, `sakura`, `shapla`, `borsha`, `jonaki`.
- Your own phrases (`~/.local/share/bornopata/phrases.tsv`), `tarikh` / `somoy` / `bar` for the date, time and weekday in Bangla, emoji beside the words they stand for (Unicode CLDR names), and `Shift+Delete` to forget a wrongly learned word.
- Packaging: a release tarball with an installer that builds nothing, the third-party license notices, and the release workflow (see `docs/plan/release.md`).
