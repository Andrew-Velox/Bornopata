#!/usr/bin/env bash
# Puts the data of the Windows installer into target/release-inputs (or the directory given as the first argument):
# words.bnd and lm.bnd are downloaded from the releases pinned in assets/dictionary/*.bnd.release and checked against
# their checksums, gloss-en.bnd is built from assets/gloss/gloss-en.tsv. Needs `gh` with a token (GH_TOKEN) that can read
# the repository's releases, and cargo.
set -euo pipefail
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
out=${1:-$repo_root/target/release-inputs}
mkdir -p "$out"

repo=${GITHUB_REPOSITORY:-$(git -C "$repo_root" remote get-url origin | sed -E 's#(git@github.com:|https://github.com/)##; s#\.git$##')}
for name in words lm; do
  pin="$repo_root/assets/dictionary/$name.bnd.release"
  tag=$(sed -n 's/^tag=//p' "$pin")
  sha=$(sed -n 's/^sha256=//p' "$pin")
  gh release download "$tag" --repo "$repo" --pattern "$name.bnd" --dir "$out" --clobber
  echo "$sha  $out/$name.bnd" | sha256sum --check
done

cargo run --release --locked --manifest-path "$repo_root/Cargo.toml" -p borno-gloss-build -- export \
  --entries "$repo_root/assets/gloss/gloss-en.tsv" -o "$out/gloss-en.bnd" \
  --license CC-BY-SA-4.0 --attribution "$(cat "$repo_root/assets/gloss/ATTRIBUTION.txt")" \
  --source https://kaikki.org/dictionary/Bengali/ --version 2026-10 --name "Bornopata English glosses"
ls -la "$out"
