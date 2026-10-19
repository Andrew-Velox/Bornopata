# CLAUDE.md

Guidance for Claude Code (claude.ai/code) in this repository.

## State of the repo

Bornopata is a cross-platform Bangla input method (phonetic, Avro-style first). The Core is platform-independent and each platform is only a shell.
The repo was imported from Qingjian, a Chinese pinyin IME (see `NOTICE.md`; `git fetch upstream` brings back any stripped file), and is being converted. Right now:

- **Done:** the Bangla Core. `Engine` takes raw Roman letters (case kept), builds candidates from the direct Avro conversion, a `WordSource` and learned choices, ranks them and commits word by word. `borno-cli` drives it end to end.
- **Not ported yet:** the Linux Server, `borno-platform`, the Fcitx5 plugin protocol, cloud prediction, glosses and the neural scorer. Their crates are listed under `exclude` in the root `Cargo.toml`, still on disk and still pinyin-shaped. `apps/linux/scripts/install.sh` is disabled until the Server returns.
- Many comments in inherited files are still Chinese. Translate a file when you change it substantially; new code and docs are English.
- Direction, decisions and open items: `docs/plan/roadmap.md`, `docs/design/bangla-engine.md`, `docs/design/bangla-reuse-plan.md`.

## Directory map

- `crates/borno-core`: the engine. `Engine` is the only entry point: `type_char`, `backspace`, `highlight`, `commit`, `candidates`, `next_words`, plus stateless `query`. The `Learner` and `LanguageModel` traits live here, implemented by the sibling crates.
- `crates/borno-transliteration`: `Transliterator` (Avro conversion through `okkhor`) and the `WordSource` seam with the `upodesh` scaffold implementation.
- `crates/borno-learning`: `FrequencyLearner` (word counts, per-input choices, personal bigram, time decay, TSV files in one directory).
- `crates/borno-lm`: `BigramModel` language model, from TSV or `.bnd`, with next-word successors.
- `crates/borno-format`: `.bnd` data container (mmap reader, zero-copy views, writer, hash index).
- `apps/cli`: `borno-cli <roman>...` types words through the engine and prints candidates, sources, scores and timings. Run it before merging ranking changes.
- Excluded until ported (see root `Cargo.toml`): `crates/borno-dictionary`, `borno-translate`, `borno-predict`, `borno-neural`, `borno-platform`, `apps/linux/server`.
- `apps/linux`: `fcitx5` (C++ plugin, default candidate panel), `scripts` (user-level install and uninstall), `server` (excluded).
- `assets/`: sample data. Every directory has a README with source and license. Data without a verified license must not ship.

`docs/` has three kinds of pages (index in `docs/README.md`): `design/` decisions, `plan/` roadmap, `notes/` engineering records.

## Commands

```bash
cargo build                                 # whole workspace
cargo test                                  # all tests; -p <crate> and a test name filter work
cargo clippy --all-targets -- -D warnings
cargo fmt --all
cargo run -p borno-cli -- ami bangla kotha  # main way to check Core behavior
cargo run -p borno-cli -- --user-dir /tmp/borno --pick 2 kotha   # choose the 2nd candidate and learn it
```

## Conventions

Architecture constraints, code layout, versions, commit messages, documentation sync and pre-commit checks are in `docs/contributing.md`, loaded with this file:

@docs/contributing.md
