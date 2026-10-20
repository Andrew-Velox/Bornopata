# Releasing

How a Linux release is cut and what the workflow (`.github/workflows/release.yml`) checks. Versions and commit rules are in [../contributing.md](../contributing.md).

## What a release contains

A GitHub release named after the tag (`v0.1.0-alpha.1`; a version with a hyphen is marked pre-release) with:

- `bornopata-<version>-linux-x86_64.tar.gz` (+ `.sha256`): the Server, the settings program, the Fcitx5 plugin, `words.bnd`, `lm.bnd`, `gloss-en.bnd`, the systemd unit, the Fcitx5 files, an `install.sh` that installs into `~/.local` without building anything, and every license and attribution (`LICENSE`, `NOTICE.md`, `THIRD-PARTY-LICENSES.txt`, the dictionary, gloss and emoji notices).
- `bornopata-<version>-source.tar.gz` (+ `.sha256`): `git archive` of the tagged commit, with a stable checksum for packagers.

The prebuilt plugin is built on Ubuntu against its Fcitx5. It may not load on a distribution with another Fcitx5; building from source is the fallback (the AUR package builds from the source archive).

## Where the dictionary comes from

`words.bnd` is not in the repository (about 20 MB, built from corpora that are not committed). It lives in a **data release** (`data-v1`, `data-v2`, ...), and `assets/dictionary/words.bnd.release` pins which one and its SHA-256. The workflow downloads it and stops if the checksum differs. A new dictionary goes to a new data release, never over an old asset, and the pin file is updated in the same commit that wants it.

The word pair model `lm.bnd` (about 2 MB) is handled the same way: its own data release (`data-lm-v1`) and `assets/dictionary/lm.bnd.release`. How it is built is in [../../assets/dictionary/README.md](../../assets/dictionary/README.md).

```bash
gh release create data-v2 --repo Andrew-Velox/Bornopata --prerelease --title "Dictionary data v2" \
  --notes "words.bnd, CC BY-SA 4.0, see assets/dictionary/README.md" ~/.local/share/bornopata/words.bnd
sha256sum ~/.local/share/bornopata/words.bnd   # into assets/dictionary/words.bnd.release
```

Always pass `--repo`: this clone also has the `upstream` (Qingjian) remote and its tags, and `gh` without `--repo` can pick that one. Never `git push --tags`; push the one tag.

## Cutting a release

1. On `main`, CI green. Run the workflow by hand first (Actions, release, Run workflow): it builds and packages everything and keeps the files as a workflow artifact, but publishes nothing.
2. Release commit: drop `-dev` from `apps/linux/server/Cargo.toml` (and `Cargo.lock`) and from `Version=` in `apps/linux/fcitx5/data/addon/bornopata.conf`; turn `## Unreleased` in `CHANGELOG.md` into `## <version> - <date>` and write what changed for users. Commit as `chore(release): <version>`.
3. Tag that commit and push only the tag: `git tag v<version> && git push origin v<version>`.
4. The workflow checks, and fails the run otherwise, that the tag is `v` + the Server version, the addon version is the same, the version has no `-dev`, the tagged commit is on `main`, `CHANGELOG.md` has a section for the version, and the dictionary checksum matches. It then runs the whole test suite, builds, packages and creates the release once; a second run for the same tag fails rather than replacing files.
5. After it, start the next version: `chore: start <next>-dev`.

A bad release is deleted (`gh release delete --repo ... --cleanup-tag v<version>`) and redone under a new version; files that were downloaded are never replaced.

## Third-party notices

`about.toml` and `about.hbs` make `THIRD-PARTY-LICENSES.txt` with `cargo about` from the Server's dependency tree (the Rust libraries are linked into the binary, so their MIT, BSD and Apache notices must travel with it). The template also carries the MIT notice of `libavrophonetic`, whose word list `upodesh` ships ([../design/bangla-reuse-plan.md](../design/bangla-reuse-plan.md)). When `deny.toml` accepts a new license, add it to `about.toml` too.

## Not yet done

- Publishing the AUR package. `apps/linux/packaging/aur/PKGBUILD` is written and was built with `makepkg` from a local copy of the source (files land under `/usr`; the Server finds `/usr/share/bornopata/words.bnd` through `XDG_DATA_DIRS`). To publish: after the tag exists, set `pkgver` (hyphen becomes underscore), run `updpkgsums`, check with `namcap` and `makepkg`, then push `PKGBUILD`, `bornopata.install` and `.SRCINFO` (`makepkg --printsrcinfo`) to the AUR git repository.
- The repository is private. A public release, and the AUR, need it public (or the files hosted elsewhere).
- Windows and macOS packages.
- Signing the files (checksums only for now).
