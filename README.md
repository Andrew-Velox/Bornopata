# Bornopata

A Bangla input method for Bangladesh, built around a platform-independent Core with thin shells per platform. Phonetic (Avro-style) typing comes first.

**Status: early development, nothing is released.** The repository was started from the Qingjian pinyin input method; the Bangla Core is not written yet and the engine in `borno-core` is still the inherited pinyin one. See [docs/plan/roadmap.md](docs/plan/roadmap.md).

## Goals

- Phonetic typing with several spelling candidates, autocorrect and next-word prediction.
- Learning from what you type, on your own machine.
- Linux (Fcitx5) first, then Windows, then macOS.

## Try the inherited tooling

```bash
cargo build
cargo test
cargo run -p borno-cli -- <input>
```

Install the Server and the Fcitx5 plugin into your home directory with `apps/linux/scripts/install.sh`; details in [docs/notes/linux-fcitx5.md](docs/notes/linux-fcitx5.md).

## Documentation

Index in [docs/README.md](docs/README.md). Contributing conventions: [docs/contributing.md](docs/contributing.md).

## License

GPL-3.0-or-later, see [LICENSE](LICENSE). Planned third-party components are MIT crates; word data licenses are tracked per directory under [assets/](assets/).
