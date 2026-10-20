# Handoff: Bangla input method, based on Qingjian

Read this first. It carries the decisions from the planning conversation that happened in `~/Projects/rust/qingjian`.
Delete or fold it into the real docs once the Bangla Core exists.

## Goal

Rebuild Qingjian (a Chinese pinyin IME) as a Bangla input method for Bangladesh.
This is a new standalone project, not a replacement inside the Qingjian repo.

## Repo state

- Local clone of `qingjian-team/qingjian` with full history, remote renamed to `upstream`. No `origin` yet.
- Next: create an empty GitHub repo, then `git remote add origin <url> && git push -u origin main`.
  Optionally disable pushes to upstream: `git remote set-url --push upstream DISABLE`.
- Pull useful shell / renderer fixes later with `git fetch upstream` and `git cherry-pick`.
- `core.hooksPath` is NOT enabled here on purpose: the upstream hooks enforce Chinese commit-message formats.
- `CLAUDE.md` and `docs/contributing.md` are still Qingjian's (Chinese, says "交流用中文"). Rewrite them in English early.

## License and naming

- GPL-3.0-or-later. Keep `LICENSE` and the copyright notices.
- `crates/bamboo-slip` and `crates/qingjian-render` are dual-licensed (GPL or commercial); the GPL side applies here.
- Rename everything (crates, binaries, logo, bundle ids). Do not reuse the Qingjian / 青简 name.
- Upstream data is Chinese-specific. Bangla word lists and corpora need their own redistributable sources, with licenses recorded per `assets/*/README.md`.

## Why Core needs a new module, not a re-skin

Pinyin IME: reading -> characters, hard part is segmenting syllables into words.
Bangla IME (Avro-style phonetic): Roman letters -> Bangla script by rules; words are space-separated, so no segmentation.
Hard parts become spelling ambiguity (o/O, t/T, sh/s, conjuncts), a phonetic-variant dictionary, and next-word prediction.

## Keep / remove

Keep: platform shells (apps/macos, apps/windows, apps/linux), `bamboo-slip`, `qingjian-render`, `apps/settings`,
`qingjian-format` (.qj), `qingjian-learning`, `qingjian-lm`, `qingjian-neural`, `qingjian-update`, `qingjian-platform`, `apps/cli`.

Remove or rewrite: `core/parser`, `shuangpin`, `zhuyin`, wubi `CodeTable`, `aux_code`, `traditional` (OpenCC),
`core/sentence` (Viterbi lattice), syllable-keyed lookup in `qingjian-dictionary`, `assets/lexicon`, `tools/gloss-gen`, `tools/corpus`.

Planned Core design: a `transliteration` module (Roman -> Bangla rules, several spelling candidates),
dictionary keyed by a phonetic skeleton for fuzzy lookup (the existing `fuzzy` module is a model),
bigram LM + learning crates for next-word prediction.

## Open decisions (ask the user)

1. Input schemes: phonetic (Avro-style) only, or also fixed layouts (Bijoy, Probhat, National)?
2. Platforms: Linux first, then Windows? Android from the start?
3. Product name and repo name (current directory name `bangla-ime` is a placeholder).
4. Differentiator vs Avro Keyboard, OpenBangla Keyboard, Ridmik, Gboard.
   Check OpenBangla first: contributing there may beat rebuilding.

## Plan

1. Rename, rewrite CLAUDE.md / contributing in English, strip down to a Linux-first skeleton (Core, CLI, Fcitx5 shell).
2. Build `bangla-core`: rules, dictionary, LM, learning.
3. Collect data: Bangla word-frequency list and a corpus with clean licenses.
4. Windows shell, then macOS.

## User's machine

Arch Linux, Hyprland (Wayland), 16 cores, 14 GB RAM, Rust 1.96.0 (matches `rust-toolchain.toml`), cmake and gcc installed.
Fcitx5 is NOT installed. Needs roughly (verify names against `apps/linux/scripts/install.sh`):

    sudo pacman -S fcitx5 fcitx5-gtk fcitx5-qt fcitx5-configtool nlohmann-json pkgconf openssl noto-fonts noto-fonts-emoji

Plus a Bangla font (Noto Sans Bengali). Hyprland needs `exec-once = fcitx5 -d` and the input-method env vars.
Baseline check before changing anything:

    cargo build -p qingjian-linux-server --locked
    cargo run -p qingjian-cli -- nihao
