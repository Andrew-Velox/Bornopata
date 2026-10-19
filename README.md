# Bornopata

A Bangla input method for Bangladesh, built around a platform-independent Core with thin shells per platform. Phonetic (Avro-style) typing comes first.

**Status: early development, nothing is released.** The repository was started from the Qingjian pinyin input method (see [NOTICE.md](NOTICE.md)). Bangla typing works end to end on Linux with Fcitx5 (build it yourself, see [docs/notes/linux-fcitx5.md](docs/notes/linux-fcitx5.md)). See [docs/plan/roadmap.md](docs/plan/roadmap.md) for what is next.

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

To type into real applications, build and install the Linux Server and the Fcitx5 plugin with `apps/linux/scripts/install.sh`; Arch packages, Hyprland setup and keys are in [docs/notes/linux-fcitx5.md](docs/notes/linux-fcitx5.md).

## Documentation

Index in [docs/README.md](docs/README.md). Contributing conventions: [docs/contributing.md](docs/contributing.md).

## License

GPL-3.0-or-later, see [LICENSE](LICENSE). Planned third-party components are MIT crates; word data licenses are tracked per directory under [assets/](assets/).
