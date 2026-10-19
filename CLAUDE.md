# CLAUDE.md

Guidance for Claude Code (claude.ai/code) in this repository.

## State of the repo

Bornopata is a cross-platform Bangla input method (phonetic, Avro-style first). The Core is platform-independent and each platform is only a shell.
The repo was imported from Qingjian, a Chinese pinyin IME (tag `qingjian-import-base`), and is being converted. Right now:

- The Linux Server and Fcitx5 shell, the CLI and the sibling crates build and pass their tests, but **`borno-core` still holds the Chinese pinyin engine**. The Bangla Core (`borno-transliteration`) has only its word source seam so far.
- Many comments and some design docs are still in Chinese. Translate a file when you change it substantially; new code and docs are English.
- Direction, decisions and what is still open: `docs/plan/roadmap.md`, `docs/design/bangla-reuse-plan.md`.

## Directory map

- `crates/borno-core`: engine facade `Engine`, with the `Translator` / `Learner` traits in the `engine` module. Currently the pinyin pipeline; to be replaced by transliteration, see the roadmap.
- `crates/borno-transliteration`: new Bangla Core piece. So far only the `WordSource` trait with a scaffold `upodesh` implementation; see `docs/design/bangla-reuse-plan.md`.
- `crates/borno-dictionary`: dictionary (TSV or `.bnd` mmap), currently keyed by pinyin syllables.
- `crates/borno-translate`: gloss table `Glossary` and word level table `LevelTable`.
- `crates/borno-learning`: on-disk user data: frequencies, user words, personal n-gram, typo table, input log, usage stats, vocabulary book.
- `crates/borno-predict`: cloud prediction `CloudPredictor` (OpenAI-compatible API) and gloss filler.
- `crates/borno-lm`: bigram language model `BigramModel`.
- `crates/borno-neural`: local character-level Transformer scorer (candle).
- `crates/borno-format`: `.bnd` data container (mmap reader, zero-copy views, writer, hash index).
- `crates/borno-platform`: shared platform layer: `Config` (TOML), extra dictionaries, Server and shell `protocol` types.
- `apps/cli`: Core validation tool: queries, per-key timing, input log replay, evaluation. Run it before merging ranking or conversion changes.
- `apps/linux`: `server` (Engine over a Unix socket), `fcitx5` (C++ plugin, default candidate panel), `scripts` (user-level install and uninstall). The Server is started by hand; see `docs/notes/linux-fcitx5.md`.
- `assets/`: sample data. Every directory has a README with source and license. Data without a verified license must not ship.

`docs/` has three kinds of pages (index in `docs/README.md`): `design/` decisions, `plan/` roadmap, `notes/` engineering records.

## Commands

```bash
cargo build                                 # whole workspace
cargo test                                  # all tests; -p <crate> and a test name filter work
cargo clippy --all-targets -- -D warnings
cargo fmt --all
cargo run -p borno-cli -- <input>...        # main way to check Core behavior
```

The Fcitx5 plugin needs Fcitx5 development packages (see `apps/linux/scripts/install.sh`); `cmake -S apps/linux/fcitx5 -B target/fcitx5 -DBUILD_TESTING=ON` builds it with its tests.

## Conventions

Architecture constraints, code layout, versions, commit messages, documentation sync and pre-commit checks are in `docs/contributing.md`, loaded with this file:

@docs/contributing.md
