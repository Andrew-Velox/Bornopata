# borno-cli

Command-line shell for the Core: no platform API, used to query candidates, inspect ranking, measure performance and replay evaluations. Check any Core change here before touching the input method.

```bash
cargo run --release -p borno-cli -- kaifa            # one query
cargo run --release -p borno-cli -- kaifa nihao      # several queries
cargo run --release -p borno-cli                     # interactive mode
```

Timing runs (`--typing`, `--replay`) must use `--release`; a debug build is ten times slower and the numbers mean nothing.

> **Inherited and pinyin-specific.** The options, data lookup order and output format still describe the pinyin pipeline and will change with the Bangla Core.
> Run `cargo run -p borno-cli -- --help` for the current flags. Without prepared data the CLI falls back to the samples in `assets/sample/`.
