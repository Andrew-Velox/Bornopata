# Changelog

Written by hand by the maintainers at release time; pull requests do not edit it.

## Unreleased

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
