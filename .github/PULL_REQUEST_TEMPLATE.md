<!-- Thanks for contributing. The maintainers read the checklist below, so filling it in saves a round trip. Strike items that do not apply. -->

## What changed

<!-- One or two sentences: the problem and how you solved it. Link issues with "Closes #number". -->

## Checklist

- [ ] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass locally
- [ ] Behavior users can notice changed (keys, candidates, settings, config or data files): the matching page under `docs/user/` is updated
- [ ] Identifiers, comments and docs are in English; new files have a `//!` header; no file over 800 lines
- [ ] Commit messages follow Conventional Commits (see [docs/contributing.md](../docs/contributing.md))
- [ ] Any new word list or corpus has its source and license recorded under `assets/<name>/README.md`
- [ ] `CHANGELOG.md` is untouched (maintainers write it at release)

## How it was verified

<!-- For bug fixes and shell behavior (keys, commit, candidate position): say how you reproduced the problem and confirmed the fix in the same application, with OS, compositor or app, and steps.
     A passing build is not verification. If you cannot reproduce it, describe the analysis in an issue or send a logging-only PR. -->
