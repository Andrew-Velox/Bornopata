# Development conventions

Read this before changing code: the architecture lines that must not be crossed, how code is organized, how versions and commit messages work, and which docs to update.
Design background is in [design/architecture.md](design/architecture.md) (inherited from Qingjian, partly Chinese) and [design/bangla-reuse-plan.md](design/bangla-reuse-plan.md); per-crate notes are in [notes/crate-notes.md](notes/crate-notes.md).

## Language

- Communicate and write code, comments, docs, logs, UI text and commit messages in English. Bangla appears only as data, test fixtures and UI strings meant for users.
- Inherited files still contain Chinese comments. Translate a file when you change it substantially. Do not translate in bulk.
- `thiserror` `#[error]` text is English lower-case without a trailing period.

## Architecture constraints (do not violate)

- **Core and platform layers are strictly separate.** `borno-core` and its sibling crates are platform-independent: dictionary, transliteration, candidate generation, ranking, learning, text transforms all belong to Core.
  A platform shell does two things only: translate system input events into Core input, and draw the frame Core returns.
  **No ranking logic, dictionary access or text transformation in a shell.** Test: replacing Fcitx5 with another framework must not require changing a line of Core.
- **Input beats everything else.** Any latency, popup or UI noise added for a secondary feature is a design error. Slow lookups (cloud prediction, glosses) must never block candidate generation; Core returns candidates first.
- **One input scheme is a setting, not a mode.** Phonetic and any future fixed layout are configuration; a new scheme must not change the established key behavior of another.
- **Word data sits behind traits.** Word suggestions come through a trait (`WordSource`, see [design/bangla-reuse-plan.md](design/bangla-reuse-plan.md)) so the list can be replaced. Do not call a dictionary crate's data directly from Core logic.
- **Data licenses are part of the architecture.** Nothing ships without its source and license recorded in `assets/<name>/README.md`. Data of unverified provenance is a development scaffold and must be marked so.

## Code organization

- **One type per file.** A struct, enum or trait and its impls live in one file; module files hold `mod` declarations, re-exports and free functions. Do not flatten a crate into `lib.rs`.
- **Submodules are directories.** Use `foo/mod.rs`, not `foo.rs` next to `foo/`.
- **Sibling files with a shared stem go into a directory**, never grouped by filename prefix: `key_event.rs` + `key_outcome.rs` becomes `key/{event,outcome}.rs`. Two or more files sharing a prefix and one concept go into a directory named after the prefix.
- **A responsibility and its private types share a directory.**
- **Split big `impl` blocks by responsibility.** One `impl Foo { ... }` per file, the struct and constructor in `mod.rs`, cross-file private methods `pub(super)`.
- **File length.** At most 800 lines, aim for 500. Tests longer than 200 lines move to `tests.rs` (or `tests/` split by topic).
- **Exact imports.** No `use super::*` or `use foo::*` (except `use super::*` in `#[cfg(test)] mod tests`). Inherited globs get replaced when you touch the file.

## Naming and comments

- Identifiers are English. New files start with a `//!` header. Fields are separated by a blank line; add `///` when the name does not explain itself.
- Comments say only what maintainers need: a file header of one or two sentences, and, in code, constraints and reasons that reading the code cannot show (platform quirks, protocol rules, past traps), in one or two sentences. No tutorial comments, no change-log comments. Longer rationale goes in `docs/`, with a one-line pointer in the code.
- No decorative divider comments (`// ====`); the pre-commit hook rejects them.

## Dependencies and configuration

- Add dependencies with `cargo add`; shared ones go to the root `[workspace.dependencies]`. Errors use `thiserror`, not `anyhow`. Logging uses the `tracing` facade.
- Shortcuts are configurable under `[shortcut]`; never hard-code key codes.
- Reuse of OpenBangla components follows [design/bangla-reuse-plan.md](design/bangla-reuse-plan.md): MIT crates as dependencies, no vendored `riti`; a single vendored `riti` layout file keeps its MPL-2.0 header.

## Versions

- `crates/*` use `version.workspace = true`; each shell under `apps/*` is released independently and pins its own `version`.
- Between releases the version carries `-dev` (for example `0.1.0-dev`); packaging appends the short git hash. A release commit drops `-dev` and gets the tag `v<version>`, then the next commit sets the next `-dev`. CI refuses tags with `-dev`.

## Commit messages

- [Conventional Commits](https://www.conventionalcommits.org/): first line `<type>(<scope>): <summary>`, type and scope lower-case, summary in English, for example `feat(transliteration): map reph before consonants`.
  - Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `style`, `revert`.
  - Scopes: a crate or shell name (`core`, `transliteration`, `platform`, `dictionary`, `learning`, `lm`, `neural`, `format`, `predict`, `translate`, `cli`, `linux`, `fcitx5`, `docs`, `ci`, `deps`, `release`); omit when a change spans many. Add `!` after the scope for incompatible changes.
  - The body explains why and the trade-offs, one point per line. `.githooks/commit-msg` rejects subjects that do not match, merge commits included.
- History before the rename (tag `qingjian-import-base`) uses Qingjian's Chinese formats and is not rewritten.

## Documentation sync

- Code wins over docs when they disagree; fix the docs. Technical direction goes in `docs/design/`, plans in `docs/plan/`, engineering records in `docs/notes/`. Keep technical content out of the README.
- Changed user-visible behavior (keys, settings, config or data files): update the matching page under `docs/user/` in the same commit (the directory will return with the first user-facing release).
- Changed a crate's implementation facts (data files, constants, generation commands): update `docs/notes/crate-notes.md`.

## Before committing

- Hooks: `.githooks/pre-commit` (divider-comment ban, fmt, clippy), `.githooks/commit-msg` (message format), `.githooks/pre-push` (whole-workspace tests). Enable once with `git config core.hooksPath .githooks`.
- Changes to ranking, conversion or correction: run `apps/cli` first.

## CI and releases

- One CI job on Linux: fmt, cargo-deny, clippy, tests, then the Fcitx5 shell build and tests. Everything runs with `--locked`. A weekly `cargo audit` runs separately and Dependabot updates actions and crates.
- The release workflow does not exist yet. It returns with the first release; until then nothing is published.

## External PRs

- Branch from main; one PR does one thing and touches one platform (Core changes get their own PR). The PR template checklist is the review standard.
- **Bug fixes and shell behavior (keys, commit, candidate position) must be verified on a real system:** reproduce first, confirm the fix in the same application, and say which OS, compositor, application and steps in the PR. A passing build is not verification.
- For features across platforms, verify the ones you have and say which you could not.
- All contributions are accepted under GPL-3.0-or-later. Third-party code or data must come with its source and license in the PR.
