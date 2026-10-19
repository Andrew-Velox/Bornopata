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
| [design/bangla-reuse-plan.md](design/bangla-reuse-plan.md) | Decision to depend on the MIT OpenBangla crates, not `riti`; license consequences; word list provenance |
| [design/architecture.md](design/architecture.md) | Inherited from Qingjian (Chinese): Core and platform split, crate structure, the `.bnd` container. Pinyin-specific parts will be rewritten |
| [design/core-internals.md](design/core-internals.md) | Inherited (Chinese): `Engine` fields, key data flow, async side paths, caches |
| [plan/roadmap.md](plan/roadmap.md) | Phases and open decisions |
| [plan/word-list-sources.md](plan/word-list-sources.md) | Candidate corpora for our own word list and their licenses |
| [notes/crate-notes.md](notes/crate-notes.md) | Inherited (Chinese): per-crate implementation facts |
| [notes/linux-fcitx5.md](notes/linux-fcitx5.md) | Inherited (Chinese): Linux panel build, protocol, install and troubleshooting |
| [notes/performance.md](notes/performance.md) | Inherited (Chinese): past performance work |
| [notes/neural-rescoring.md](notes/neural-rescoring.md) | Inherited (Chinese): local neural rescoring |
| [notes/upodesh-provenance-inquiry.md](notes/upodesh-provenance-inquiry.md) | Draft question to the upodesh maintainers, not sent |
