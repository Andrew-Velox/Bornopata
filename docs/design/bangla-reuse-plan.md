# Reusing OpenBangla components

Decided 2026-10-19 after reading OpenBangla Keyboard and its engine library `riti`. This page exists so the decision survives context loss.

## Decision

- **Depend on the MIT crates** `okkhor` (Roman to Bangla rules), `poriborton` and `upodesh` (word suggestions) from crates.io.
- **Write our own thin `borno-transliteration` layer** on top: key and text events in, candidates out, autocorrect, learning, language model and prediction.
- **Do not depend on or vendor `riti`.** It is not on crates.io, is MPL-2.0, is at version 0.1.0 with no API promise, and its API is keystroke-driven (raw key codes), which does not match the Core interface here.
- If we ever need a `riti` fixed-layout file (Probhat and similar), vendor only that file and keep its MPL-2.0 header.

## How `upodesh` is pulled in

`upodesh` 0.4.0 on crates.io pins `once_cell = "~1.20"`, which conflicts with `candle-metal-kernels` (`^1.21`) in this workspace. Upstream master (unreleased 0.4.1) relaxes it to `"1"`,
so `borno-transliteration` depends on `upodesh` as a git dependency pinned to commit `fc61225ebd64fc142305e7c4b31d9e4e2b9ec5e4`, and `deny.toml` allows that one git source.
Switch back to crates.io when 0.4.1 is published. The seam is `borno_transliteration::WordSource`; `UpodeshWordSource` is the scaffold implementation and `FixedWordSource` an in-memory one for tests.

## Findings behind it

- `riti` ([OpenBangla/riti](https://github.com/OpenBangla/riti)): about 5.3k lines, MPL-2.0. Public API is `Config` plus `Context::get_suggestion_for_key(key, modifier, selection)`, `candidate_committed`, `backspace_event`, `finish_input_session`. The only learning is a remembered selection per word.
- OpenBangla Keyboard itself is GPL-3.0 and Linux only (IBus and fcitx5). Nothing in its issues or releases mentions next-word prediction or a Windows or macOS port, so our differentiators (prediction, richer learning, other platforms) are real.
- `okkhor`, `poriborton`, `upodesh` and `emojicon` are all MIT.

## License consequences

- MIT dependencies are compatible with this repo's GPL-3.0-or-later. No license change.
- Any vendored `riti` file stays MPL-2.0 under its own header. MPL file-level copyleft is allowed inside a GPL project.
- The dual-licensed crates (`bamboo-slip`, the renderer, if they return) must not depend on or include MPL code or any word-list data. Transliteration data lives in Core only.

## Word list provenance (blocking for release)

`upodesh` ships a word list (`data/dictionary.json`, `source-words.txt`) and `riti` ships `dictionary.json` and `autocorrect.json`. Neither repo states where the words came from. The lineage points at Avro Keyboard via `libavrophonetic`, but that is an inference.

- `upodesh` goes behind a trait (`WordSource`, word suggestions for a Roman input) so the list can be swapped without touching Core.
- Until replaced, `upodesh`'s word list is a **development scaffold only: provenance unverified, must be replaced before release.** Say so in `assets/*/README.md` wherever it appears, and in any release checklist.
- Our own word list and frequencies come from a clearly redistributable corpus. Candidates and their license implications: [plan/word-list-sources.md](../plan/word-list-sources.md).
- A draft question to the maintainers (not sent) is in [notes/upodesh-provenance-inquiry.md](../notes/upodesh-provenance-inquiry.md).
