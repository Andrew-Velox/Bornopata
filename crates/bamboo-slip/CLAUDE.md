# CLAUDE.md

The self-drawn UI engine `bamboo-slip`. Its public API is re-exported from `lib.rs`; the modules are private. Code layout, comments and commit
messages follow the repository's `docs/contributing.md`.

## Boundary

- No application concepts inside the engine (candidates, typed letters, theme file formats, frame data, config keys). Users convert their own
  data into the engine's types (`Scene`, `Visual`, `Effect`, `Keyframes`, `TextStyle`) and pass them in; anything the user must customize goes
  through a trait such as `Icon`.
- No dependency on any `borno-*` crate.
- The scene tree holds pixels only; the user multiplies points by the scale factor.
- Only the drawing engine is kept from the upstream crate (scene, layout, text, fonts, animation, SVG): the window host, the settings widgets and the
  macOS and Windows font code were removed.
- After a change run this crate's tests and `cargo test -p borno-render`, whose rendering tests are the regression check.

## Commands

```bash
cargo test -p bamboo-slip
cargo clippy -p bamboo-slip --all-targets -- -D warnings
```
