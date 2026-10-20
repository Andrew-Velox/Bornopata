# Text shaping and the vendored cosmic-text

## The problem

`bamboo-slip` shapes text with `cosmic-text`, which shapes with `harfrust` (a Rust port of HarfBuzz). `cosmic-text` 0.19.0, the latest release, pins `harfrust` 0.5, and `harfrust` 0.5 and 0.6 mis-form Bengali conjuncts with Noto Sans Bengali: a letter followed by a ra-phala or a ba-phala came out as the plain letter, a visible hasanta and a full-width র, and half forms before them were missing. Words like প্রস্তাবিত, নিয়ন্ত্রণ, স্বয়ংক্রিয় and ক্রিকেট looked broken, in the candidate window and in the settings program alike. Real HarfBuzz (Pango, `uharfbuzz`) forms them correctly; `harfrust` 0.14 does too.

The font itself is fine. One thing that looks like a fault is not: Noto Sans Bengali draws য় as `ya-beng` plus a nukta mark by design.

## What is vendored

`vendor/cosmic-text` is cosmic-text 0.19.0 (MIT OR Apache-2.0, licenses kept) with two changes, wired in by `[patch.crates-io]` in the root `Cargo.toml`:

- `Cargo.toml` asks for `harfrust` 0.14 instead of 0.5, and drops the tests, benches and dev-dependencies of the published crate.
- `src/font/mod.rs` and `src/shape.rs` use the 0.14 API: a `harfrust::Font` instance at the requested weight, a `ShaperFont` built per shaping call (it reuses cached layout data, so shaping stays at about 6 µs a word), `ShapePlan` and `shape`. Nothing else in `cosmic-text` changed.

`crates/bamboo-slip/src/text/tests.rs` fails on the old shaper (a conjunct must be narrower than its letters side by side) and skips itself where no Noto Sans Bengali is installed.

## Removing it

When a `cosmic-text` release depends on a `harfrust` that shapes Bengali correctly (check with a test like the one above, and with `uharfbuzz` for the reference glyphs), delete `vendor/cosmic-text` and the `[patch.crates-io]` section and bump the dependency.
