#!/usr/bin/env bash
# Builds the English gloss file from the table in assets/gloss: build-gloss.sh OUTPUT_FILE [--debug]
set -euo pipefail
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
output=${1:?usage: build-gloss.sh OUTPUT_FILE [--debug]}
profile=release
[[ "${2:-}" != --debug ]] || profile=debug
cargo_output=$(python3 -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' "${CARGO_TARGET_DIR:-$repo_root/target}")
args=(build --manifest-path "$repo_root/Cargo.toml" --target-dir "$cargo_output" -p borno-gloss-build --locked)
[[ "$profile" != release ]] || args+=(--release)
cargo "${args[@]}"
mkdir -p "$(dirname "$output")"
"$cargo_output/$profile/borno-gloss-build" export --entries "$repo_root/assets/gloss/gloss-en.tsv" -o "$output" \
  --license CC-BY-SA-4.0 --attribution "$(cat "$repo_root/assets/gloss/ATTRIBUTION.txt")" \
  --source https://kaikki.org/dictionary/Bengali/ --version 2026-10 --name "Bornopata English glosses"
