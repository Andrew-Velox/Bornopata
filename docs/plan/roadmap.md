# Roadmap

Updated 2026-10-20.

## Decisions made

- Product name: **Bornopata**; crates are `borno-*`.
- First input scheme: phonetic (Avro-style) only. Fixed layouts (Bijoy, Probhat, National) are a later configuration option.
- Platforms: Linux (Fcitx5) first, then Windows. macOS and Android are not planned yet.
- Reuse: depend on the MIT crates `okkhor`, `poriborton`, `upodesh`; write our own thin layer; no `riti`. See [design/bangla-reuse-plan.md](../design/bangla-reuse-plan.md).
- Word data: `upodesh`'s list is a scaffold with unverified provenance and must be replaced before release. Candidates: [word-list-sources.md](word-list-sources.md).
- Engine shape: [design/bangla-engine.md](../design/bangla-engine.md). Linux Server, plugin and keys: [notes/linux-fcitx5.md](../notes/linux-fcitx5.md).
- The Bangla/English toggle is `[shortcut] toggle`, default `ctrl+alt+space`. Enter commits the word and passes through. Digits become Bangla digits unless `bangla_digits = false`.
- Differentiators over OpenBangla Keyboard: next-word prediction, richer on-device learning, Windows and macOS shells.

## Done

- Repository imported from Qingjian, stripped, renamed, docs and CI rewritten in English.
- Bangla engine in `borno-core`; `borno-transliteration`, `borno-learning`, `borno-lm`, `borno-cli`.
- The pinyin-only crates (`borno-dictionary`, `borno-translate`, `borno-predict`, `borno-neural`) are dropped.
- `borno-platform`: slim config and the new wire protocol (version 1).
- `borno-linux-server`: `Router` with per-context engines over shared learning, Unix socket IPC, systemd user unit.
- `borno-dictionary`: normalization, phonetic key, the FST-indexed `.bnd` dictionary, prefix completion; `tools/dict-build` builds it from Wikipedia and Common Voice (413k words, 92% top-1 on the 142-spelling set). The engine ranks by frequency and shows completions after what was typed.
- Fcitx5 plugin on the default panel, install and uninstall scripts, CI building and testing server and plugin together.

## Next

1. Use it daily on Arch/Hyprland and collect what is wrong: key behavior, candidate order, the next-word hint, applications that misbehave. Fix in small commits.
2. A bigram model from the same corpora (`borno-lm` takes TSV counts) for context ranking and next-word suggestions; retune the ranking on real typing data instead of the hand-written evaluation set. Decide whether to drop the `upodesh` fallback and its git pin once the dictionary covers enough; ship `words.bnd` with the package (CC BY-SA 4.0, see [assets/dictionary/README.md](../../assets/dictionary/README.md)).
3. Spelling variants the key cannot reach (`moddhe` for মধ্যে, `porjonto` for পর্যন্ত): extra key rules or a small list of alternative spellings.
4. Settings: a settings UI or at least `fcitx5-configtool` integration, config reload without restart.
5. Packaging for Arch (AUR) and a release workflow.
6. Windows shell, then macOS.

## Open

- Whether and how to send the question to the `upodesh` / OpenBangla maintainers (draft in [notes/upodesh-provenance-inquiry.md](../notes/upodesh-provenance-inquiry.md)).
- The settings program and self-drawn renderer (`bamboo-slip`, `qingjian-render`) were removed; recover them from `upstream` if a candidate window or settings UI is needed beyond Fcitx5's default panel.
- `upodesh` is a git dependency until 0.4.1 is published to crates.io, and remains the fallback where the dictionary has no spelling of the input.
- The dictionary's license (CC BY-SA 4.0 because of the Wikipedia counts) needs a place in the installer and the About text before anything is released.
- Ported C++ tests dropped `multiplex` and `reentrant` (they tested removed pinyin features); add equivalents if connection sharing or reentrancy regress.
- Next-word suggestions are shown after Space and taken with Tab or a click; whether to take them with a number key or arrow keys is open.
