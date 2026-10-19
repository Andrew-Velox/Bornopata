# Changelog

Written by hand by the maintainers at release time; pull requests do not edit it.

## Unreleased

- Project started as Bornopata, imported from the Qingjian pinyin input method (history up to tag `qingjian-import-base`).
- Stripped to Core, CLI and the Linux Server with the Fcitx5 shell.
- Real word data: `tools/dict-build` builds a 413k-word dictionary from Bengali Wikipedia and Common Voice counts; typing shows frequency-ranked spellings and prefix completions (`bangl` gives বাংলা, বাংলাদেশ ...).
- Crates renamed to `borno-*`; the data container extension is now `.bnd` and the model extension `.bnm`.
