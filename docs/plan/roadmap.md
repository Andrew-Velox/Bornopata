# Roadmap

Updated 2026-10-20.

## Decisions made

- Product name: **Bornopata**; crates are `borno-*`.
- First input scheme: phonetic (Avro-style) only. Fixed layouts (Bijoy, Probhat, National) are a later configuration option.
- Platforms: Linux (Fcitx5) first, then Windows. macOS and Android are not planned yet.
- Reuse: depend on the MIT crates `okkhor`, `poriborton`, `upodesh`; write our own thin layer; no `riti`. See [design/bangla-reuse-plan.md](../design/bangla-reuse-plan.md).
- Word data: `upodesh`'s list is a scaffold with unverified provenance and must be replaced before release. Candidates: [word-list-sources.md](word-list-sources.md).
- Engine shape: [design/bangla-engine.md](../design/bangla-engine.md).
- Differentiators over OpenBangla Keyboard: next-word prediction, richer on-device learning, Windows and macOS shells.

## Done

- Repository imported from Qingjian, stripped, renamed, docs and CI rewritten in English.
- `borno-transliteration`: `Transliterator` over `okkhor`, `WordSource` seam with the `upodesh` scaffold.
- Bangla engine in `borno-core`: raw Roman composition with case kept, candidates (direct, dictionary, learned), ranking, commit, next words. The pinyin pipeline is deleted.
- `borno-learning` rewritten around the new `Learner`: word counts, per-input choices, personal bigram, decay, TSV persistence.
- `borno-lm` implements the new `LanguageModel` with next-word successors.
- `borno-cli` rewritten: types words through the engine, shows sources, scores, timings, learns with `--user-dir`.

## Still pinyin-shaped (next phase)

These are excluded from the workspace and still describe the pinyin engine:

- **`apps/linux/server`**: calls about 75 `Engine` methods that no longer exist (pinyin modes, aux code, annotations, cloud and neural rescoring, vocabulary), and builds the Chinese engine in `assembly/`. Needs a rewrite against the new `Engine`.
- **`crates/borno-platform`**: `Config` has shuangpin, zhuyin, traditional, aux-code and fuzzy sections; the `protocol` frames carry pinyin preedit segments, annotations and Chinese/English mode state. Keep the transport, key event and socket parts; replace the frame and config content.
- **`apps/linux/fcitx5`** (C++): speaks that protocol, so it follows the frame change (preedit, candidate list, commit). Its tests use a real Server binary and are off in CI until the Server returns. Mode keys and Chinese punctuation handling go.
- **`apps/linux/scripts/install.sh`**: disabled until there is a Server to build.
- **`crates/borno-dictionary`**: syllable-keyed lookup. To be replaced by a word-frequency dictionary behind `WordSource`, keyed by a phonetic skeleton.
- **`borno-predict`, `borno-translate`, `borno-neural`**: cloud prediction, glosses and the character-level scorer. Decide per crate whether Bornopata wants them; none is needed for the first release.

## Next

1. Port the Linux Server, `borno-platform` protocol and the Fcitx5 plugin to the new `Engine`; re-enable the Fcitx5 CI steps and `install.sh`. Type Bangla into a real application.
2. Own word list and bigram model from a clearly licensed corpus ([word-list-sources.md](word-list-sources.md)); record sources under `assets/`. Replace the `upodesh` scaffold. Then use real frequencies in ranking and retune with the CLI.
3. Conjunct and spelling variants beyond the single Avro spelling: a phonetic-skeleton dictionary lookup (the old `fuzzy` idea, in `git fetch upstream`).
4. Config file for `borno-platform` (scheme, shortcuts, learning on or off), settings UI decision.
5. Windows shell, then macOS.

## Open

- Whether and how to send the question to the `upodesh` / OpenBangla maintainers (draft in [notes/upodesh-provenance-inquiry.md](../notes/upodesh-provenance-inquiry.md)).
- Release and packaging workflow: removed with the strip, to be rebuilt for the first release.
- The settings program and self-drawn renderer (`bamboo-slip`, `qingjian-render`) were removed; recover them from `upstream` if a candidate window or settings UI is needed beyond Fcitx5's default panel.
- `upodesh` is a git dependency until 0.4.1 is published to crates.io.
