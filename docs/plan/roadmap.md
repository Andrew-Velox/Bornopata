# Roadmap

Updated 2026-10-20.

## Decisions made

- Product name: **Bornopata**; crates are `borno-*`.
- First input scheme: phonetic (Avro-style) only. Fixed layouts (Bijoy, Probhat, National) are a later configuration option.
- Platforms: Linux (Fcitx5) first, then Windows. macOS and Android are not planned yet.
- Reuse: depend on the MIT crates `okkhor`, `poriborton`, `upodesh`; write our own thin `borno-transliteration` layer; no `riti`. See [design/bangla-reuse-plan.md](../design/bangla-reuse-plan.md).
- Word data: `upodesh`'s list is a scaffold with unverified provenance and must be replaced before release. Candidates: [word-list-sources.md](word-list-sources.md).
- Differentiators over OpenBangla Keyboard: next-word prediction, richer on-device learning, Windows and macOS shells.

## Done

- `borno-transliteration` created with the `WordSource` seam and a scaffold `upodesh` implementation (git-pinned, see the reuse plan).
- Repository imported from Qingjian, stripped to Core, CLI and Linux Server with the Fcitx5 shell, renamed, docs and CI rewritten in English.

## Next

1. Grow `borno-transliteration` (the `WordSource` trait over `upodesh` exists): `okkhor` conversion for the phonetic rules, an adapter from Core input events.
2. Replace the pinyin parts of `borno-core` (`parser`, `shuangpin`, `zhuyin`, `sentence` lattice, `traditional`, aux code) and the syllable-keyed lookup in `borno-dictionary`.
3. Rewrite the CLI and the Server protocol against the new Core; update the Fcitx5 shell labels and tests.
4. Build our own word list and bigram model from a clearly licensed corpus; record sources under `assets/`.
5. Hook up learning and next-word prediction (`borno-learning`, `borno-lm`).
6. Windows shell (history in tag `qingjian-import-base`), then macOS.

## Open

- Whether and how to send the question to the `upodesh` / OpenBangla maintainers (draft in [notes/upodesh-provenance-inquiry.md](../notes/upodesh-provenance-inquiry.md)).
- GitHub repository name and URL (a placeholder was dropped from `Cargo.toml`).
- Release and packaging workflow: removed with the strip, to be rebuilt for the first release.
- The settings program and self-drawn renderer (`bamboo-slip`, `qingjian-render`) were removed from the tree; recover them from the tag if a candidate window or settings UI is needed beyond Fcitx5's default panel.
