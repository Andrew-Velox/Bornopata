# Bornopata

A Bangla input method for Bangladesh, built around a platform-independent Core with thin shells per platform. Phonetic (Avro-style) typing comes first.

**Status: early development, nothing is released.** The repository was started from the Qingjian pinyin input method (see [NOTICE.md](NOTICE.md)). The Bangla Core works end to end in the CLI; the Linux Server and Fcitx5 plugin are not ported to it yet. See [docs/plan/roadmap.md](docs/plan/roadmap.md).

## Goals

- Phonetic typing with several spelling candidates, autocorrect and next-word prediction.
- Learning from what you type, on your own machine.
- Linux (Fcitx5) first, then Windows, then macOS.

## Try it

```bash
cargo build
cargo test
cargo run -p borno-cli -- ami bangla kotha
```

The CLI prints each word's candidates with their source (direct, dictionary or learned), score and timings, then the committed text and next-word suggestions. `--user-dir DIR` keeps what you choose between runs; `--pick N` commits the Nth candidate.

Typing into real applications needs the Fcitx5 plugin and the Linux Server, which return once they are ported (roadmap).

## Documentation

Index in [docs/README.md](docs/README.md). Contributing conventions: [docs/contributing.md](docs/contributing.md).

## License

GPL-3.0-or-later, see [LICENSE](LICENSE). Planned third-party components are MIT crates; word data licenses are tracked per directory under [assets/](assets/).
