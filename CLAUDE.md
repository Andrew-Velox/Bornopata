# CLAUDE.md

Guidance for Claude Code (claude.ai/code) in this repository.

## State of the repo

Bornopata is a cross-platform Bangla input method (phonetic, Avro-style first). The Core is platform-independent and each platform is only a shell.
The repo was imported from Qingjian, a Chinese pinyin IME (see `NOTICE.md`; `git fetch upstream` brings back any stripped file), and is being converted. Right now:

- **Done:** the Bangla Core and the Linux path. `Engine` takes raw Roman letters (case kept), builds candidates from the direct Avro conversion, a `WordSource` and learned choices, ranks them and commits word by word. `borno-cli` drives it. `borno-linux-server` serves it over a Unix socket to the Fcitx5 plugin, so Bangla can be typed in real applications (`docs/notes/linux-fcitx5.md`).
- **Not done:** our own word list (the `upodesh` list is a scaffold with unverified provenance), packaging and releases, Windows and macOS shells.
- Many comments in inherited files are still Chinese. Translate a file when you change it substantially; new code and docs are English.
- Direction, decisions and open items: `docs/plan/roadmap.md`, `docs/design/bangla-engine.md`, `docs/design/bangla-reuse-plan.md`.

## Directory map

- `crates/borno-core`: the engine. `Engine` is the only entry point: `type_char`, `backspace`, `highlight`, `commit`, `candidates`, `next_words`, plus stateless `query`. The `Learner` and `LanguageModel` traits live here, implemented by the sibling crates.
- `crates/borno-dictionary`: Bengali word normalization, the phonetic `key`, the FST-indexed `.bnd` dictionary with prefix completion (`Dictionary`, `DictionaryBuilder`) and `DictionarySource`, the main `WordSource`.
- `crates/borno-transliteration`: `Transliterator` (Avro conversion through `okkhor`) and the `WordSource` seam with the `upodesh` scaffold implementation.
- `crates/borno-learning`: `FrequencyLearner` (word counts, per-input choices, personal bigram, time decay, TSV files in one directory).
- `crates/borno-lm`: `BigramModel` language model, from TSV or `.bnd`, with next-word successors.
- `crates/borno-format`: `.bnd` data container (mmap reader, zero-copy views, writer, hash index).
- `apps/cli`: `borno-cli <roman>...` types words through the engine and prints candidates, sources, scores and timings; `--dict` picks a dictionary, `--eval FILE` measures ranking. Run it before merging ranking changes.
- `tools/dict-build`: `borno-dict-build` counts Bengali words in corpus text (wiki XML included) and writes the dictionary. Corpora are never committed.
- `crates/borno-platform`: the config file (`[general]`, `[shortcut]`) and the Server and shell wire protocol (`protocol`).
- `crates/bamboo-slip`: the self-drawn UI engine (scene tree, layout, text shaping, animation to a bitmap), from Qingjian, GPL side of its dual license.
- `crates/borno-render`: the candidate window renderer: `Frame` + theme to a bitmap. Themes `bornopata` and `sakura` are in `themes/`; `cargo run -p borno-render --example preview` draws them to PNG.
- `crates/borno-panel`: the Wayland window: a layer-shell surface on its own thread, popup placement, Hyprland geometry. `examples/show.rs` puts it on a real screen.
- `apps/linux`: `server` (`Router` input logic, Unix socket IPC), `fcitx5` (C++ plugin; draws the Fcitx5 panel only when the Server's own window is not available), `scripts` (user-level install and uninstall), `systemd` (user unit).
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
