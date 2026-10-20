# Notice

Bornopata is derived from [Qingjian](https://github.com/qingjian-team/qingjian), a cross-platform pinyin input method
released under the GNU General Public License v3.0 or later. The Core crates, the learning and language-model code,
the `.bnd` data container and the Linux Fcitx5 shell began as Qingjian code and carry its copyright.
Copyright remains with the Qingjian contributors, whose names are in the history of the upstream repository.

This repository starts from a fresh history, so the upstream repository is the record of who wrote what.
Bornopata is distributed under the same license, GPL-3.0-or-later; see `LICENSE`.

`crates/bamboo-slip` (with its widget kit, interaction layer and window host, restored from Qingjian for the settings program) and `crates/borno-render` (from Qingjian's `qingjian-render`) were dual-licensed upstream under the GPL or a
commercial license, as their `LICENSE.md` files say. Bornopata uses and distributes them under the GPL side only and cannot offer the
commercial license. The themes' pictures are new; none of Qingjian's theme artwork is included.

`apps/linux/fcitx5/protocol/input-method-unstable-v2.xml` is the Wayland protocol description of the input-method popup, copied unchanged from the
`wayland-protocols-misc` crate; its own copyright and permission notice (MIT) is in the file.

Third-party components are listed with their licenses in `docs/design/bangla-reuse-plan.md` and in each `assets/*/README.md`.

The emoji table (`assets/emoji/emoji.tsv`, built into the Server) is derived from Unicode's `emoji-test.txt` and the CLDR annotations, Copyright © 1991-2026 Unicode, Inc., used under the Unicode License v3; the notice is in `assets/emoji/LICENSE-unicode.txt` and is installed with the program.
