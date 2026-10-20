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

## Word list provenance (decided 2026-10-20: accepted, with the lineage recorded)

`upodesh` ships a word list (`data/dictionary.json`, `source-words.txt`) and `riti` ships `dictionary.json` and `autocorrect.json`. Neither repo states where the words originally came from.

What was checked (2026-10-20):

- `upodesh` is MIT (Copyright 2025 OpenBangla). Its README says it is inspired by `libavrophonetic` and thanks its authors, Mehdi Hasan Khan and Tahmid Sadik.
- `libavrophonetic` (Go) is MIT (Copyright 2021 Mehdi Hasan Khan) and carries the same `data/source-words.txt`; its README now points to `upodesh` as the better implementation. So `upodesh`'s list is the `libavrophonetic` list. That lineage and the credit are about the code and the method; neither repo says where the words themselves first came from (probably the Avro Keyboard dictionary, which is an inference).

Decision of the project owner: use the `upodesh` list as the fallback under its MIT license, accepting that the original source of the words is not stated. The risk is low (a list of common Bangla words, published under MIT by the maintainers of the GPL-3.0 OpenBangla Keyboard, whose input method it serves) but not zero, so:

- `upodesh` stays behind a trait (`WordSource`), so the list can be swapped or dropped without touching Core. Our own dictionary answers first and `upodesh` only fills in where it has no spelling.
- The MIT notices of `upodesh` and `libavrophonetic` ship with every binary release (third-party notices in the release checklist).
- `riti`'s data stays excluded: nothing of it is used.
- The question to the maintainers (draft: [notes/upodesh-provenance-inquiry.md](../notes/upodesh-provenance-inquiry.md), not sent) can still be sent as a courtesy; if the answer is bad, the fallback is dropped.
- Our own word list and frequencies come from a clearly redistributable corpus. Candidates and their license implications: [plan/word-list-sources.md](../plan/word-list-sources.md).
