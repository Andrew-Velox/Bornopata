#!/usr/bin/env bash
# Installs the Server, the Fcitx5 plugin and a systemd user unit into a user directory.
set -euo pipefail
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
install_prefix="$HOME/.local"
build_profile=release
cmake_type=Release
while (($#)); do
  case "$1" in
    --prefix) install_prefix=${2:?--prefix needs a path}; shift 2 ;;
    --debug) build_profile=debug; cmake_type=Debug; shift ;;
    --help) echo 'usage: install.sh [--prefix ABSOLUTE_USER_DIR] [--debug]'; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ "$install_prefix" = /* && "$install_prefix" != / ]] || { echo 'the install prefix must be an absolute path other than /' >&2; exit 2; }
missing=()
for tool in cargo cmake c++ pkg-config python3; do
  command -v "$tool" >/dev/null 2>&1 || missing+=("$tool")
done
if command -v pkg-config >/dev/null 2>&1; then
  for module in nlohmann_json Fcitx5Core Fcitx5Utils Fcitx5Config; do
    pkg-config --exists "$module" || missing+=("$module")
  done
fi
if ((${#missing[@]})); then
  echo "missing dependencies: ${missing[*]}" >&2
  echo 'Arch: sudo pacman -S fcitx5 cmake pkgconf nlohmann-json' >&2
  echo 'Debian / Ubuntu: sudo apt install cmake g++ pkg-config python3 libfcitx5core-dev libfcitx5utils-dev libfcitx5config-dev nlohmann-json3-dev; Rust: https://rustup.rs' >&2
  exit 1
fi
install_prefix=$(python3 -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' "$install_prefix")
cargo_output=$(python3 -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' "${CARGO_TARGET_DIR:-$repo_root/target}")
cargo_args=(build --manifest-path "$repo_root/Cargo.toml" --target-dir "$cargo_output" -p borno-linux-server --locked)
[[ "$build_profile" != release ]] || cargo_args+=(--release)
cargo "${cargo_args[@]}"
# The English glosses beside the candidates: built from the table in assets/gloss and installed next to the dictionary.
gloss_args=(build --manifest-path "$repo_root/Cargo.toml" --target-dir "$cargo_output" -p borno-gloss-build --locked)
[[ "$build_profile" != release ]] || gloss_args+=(--release)
cargo "${gloss_args[@]}"
gloss_file="$repo_root/target/gloss-install/gloss-en.bnd"
mkdir -p "$(dirname "$gloss_file")"
"$cargo_output/$build_profile/borno-gloss-build" export --entries "$repo_root/assets/gloss/gloss-en.tsv" -o "$gloss_file" \
  --license CC-BY-SA-4.0 --attribution "$(cat "$repo_root/assets/gloss/ATTRIBUTION.txt")" \
  --source https://kaikki.org/dictionary/Bengali/ --version 2026-10 --name "Bornopata English glosses"
cmake_output="$repo_root/target/fcitx5-install-$build_profile"
cmake -S "$repo_root/apps/linux/fcitx5" -B "$cmake_output" "-DCMAKE_BUILD_TYPE=$cmake_type" -DBUILD_TESTING=OFF
cmake --build "$cmake_output" --parallel "${CMAKE_BUILD_PARALLEL_LEVEL:-2}"
python3 "$repo_root/apps/linux/scripts/files.py" install "$install_prefix" "$repo_root" "$cargo_output/$build_profile/borno-linux-server" "$cmake_output/bornopata.so" "$gloss_file"
printf 'Glosses: %s\n' "$(cat "$repo_root/assets/gloss/ATTRIBUTION.txt")"
printf 'Installed.\nStart the Server: systemctl --user daemon-reload && systemctl --user enable --now bornopata   (or run %s/bin/borno-linux-server yourself)\nRestart Fcitx5, then add "Bornopata" in the configuration tool (untick "Only show current language").\n' "$install_prefix"
