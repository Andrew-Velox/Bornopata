# Changelog

Written by hand by the maintainers at release time; pull requests do not edit it.

## Unreleased

- Project started as Bornopata, imported from the Qingjian pinyin input method (history up to tag `qingjian-import-base`).
- Stripped to Core, CLI and the Linux Server with the Fcitx5 shell.
- Real word data: `tools/dict-build` builds a 413k-word dictionary from Bengali Wikipedia and Common Voice counts; typing shows frequency-ranked spellings and prefix completions (`bangl` gives বাংলা, বাংলাদেশ ...).
- Crates renamed to `borno-*`; the data container extension is now `.bnd` and the model extension `.bnm`.
- Wayland-native applications (Brave, Firefox, GTK, Qt) show the candidate window at the text cursor through the input-method popup surface.
- English glosses beside the candidates (`n. rice`), from Wiktionary (CC BY-SA 4.0), for part of the 5,000 most frequent words.
- Five candidate-window themes: `bornopata`, `sakura`, `shapla`, `borsha`, `jonaki`.
- Your own phrases (`~/.local/share/bornopata/phrases.tsv`), `tarikh` / `somoy` / `bar` for the date, time and weekday in Bangla, emoji beside the words they stand for (Unicode CLDR names), and `Shift+Delete` to forget a wrongly learned word.
- Packaging: a release tarball with an installer that builds nothing, the third-party license notices, and the release workflow (see `docs/plan/release.md`).
