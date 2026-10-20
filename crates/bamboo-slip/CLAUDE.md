# CLAUDE.md

The self-drawn UI engine `bamboo-slip`. Its public API is re-exported from `lib.rs`; the modules are private. Code layout, comments and commit
messages follow the repository's `docs/contributing.md`.

## Boundary

- No application concepts inside the engine (candidates, typed letters, theme file formats, frame data, config keys). Users convert their own
  data into the engine's types (`Scene`, `Visual`, `Effect`, `Keyframes`, `TextStyle`) and pass them in; anything the user must customize goes
  through a trait such as `Icon`.
- No dependency on any `borno-*` crate.
- The scene tree holds pixels only; the user multiplies points by the scale factor.
- The drawing engine (scene, layout, text, fonts, animation, SVG) is the core. The widget kit (`widgets/`), the interaction layer (`ui/`, `app/`) and,
  behind the `host` feature, the winit window host (`host/`) were restored from Qingjian for the settings program. The macOS and Windows font code
  is not here until those shells exist.
- After a change run this crate's tests and `cargo test -p borno-render`, whose rendering tests are the regression check.

## Commands

```bash
cargo test -p bamboo-slip
cargo clippy -p bamboo-slip --all-targets -- -D warnings
```
