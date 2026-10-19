# docs

The README only introduces the project; technical content lives here, in three kinds of page.

| Directory | Contents |
|---|---|
| [design/](design/) | Design: what it is and why it was decided that way |
| [plan/](plan/) | Plans: what comes next and in what order |
| [notes/](notes/) | Engineering records: performance history, post-mortems, traps, per-crate implementation facts |

`docs/user/` (user-facing documentation) does not exist yet and returns with the first release.

| File | Contents |
|---|---|
| [contributing.md](contributing.md) | Conventions: architecture constraints, code layout, versions, commit messages, doc sync, checks, external PRs (CLAUDE.md loads it) |
| [design/candidate-window.md](design/candidate-window.md) | The self-drawn candidate window: parts, why a layer-shell overlay, placement, fonts, themes |
| [design/popup-surface.md](design/popup-surface.md) | The candidate window inside the compositor-placed input-method popup: spike result, popup channel, limits |
| [design/bangla-reuse-plan.md](design/bangla-reuse-plan.md) | Decision to depend on the MIT OpenBangla crates, not `riti`; license consequences; word list provenance |
| [design/bangla-engine.md](design/bangla-engine.md) | The Bangla engine: composition, candidates, ranking, learning, next-word suggestions |
| [design/architecture.md](design/architecture.md) | Inherited from Qingjian (Chinese): Core and platform split, crate structure, the `.bnd` container. Pinyin-specific parts will be rewritten |
| [plan/roadmap.md](plan/roadmap.md) | Phases and open decisions |
| [../assets/dictionary/README.md](../assets/dictionary/README.md) | Dictionary build, sources, license and measurements |
| [plan/word-list-sources.md](plan/word-list-sources.md) | Corpora considered for the word list, their licenses, and which are used |
| [../assets/dictionary/README.md](../assets/dictionary/README.md) | Dictionary build, sources, license and measurements |
| [notes/crate-notes.md](notes/crate-notes.md) | Inherited (Chinese): per-crate implementation facts |
| [notes/linux-fcitx5.md](notes/linux-fcitx5.md) | Linux Server and Fcitx5 plugin: Arch packages, build, install, enabling on Hyprland, keys, config, protocol, tests |
| [notes/upodesh-provenance-inquiry.md](notes/upodesh-provenance-inquiry.md) | Draft question to the upodesh maintainers, not sent |
