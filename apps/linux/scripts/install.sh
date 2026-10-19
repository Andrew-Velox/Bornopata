#!/usr/bin/env bash
# Installs the Server and the Fcitx5 plugin into a user directory. The Server is started by hand.
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
  for module in openssl nlohmann_json Fcitx5Core Fcitx5Utils Fcitx5Config; do
    pkg-config --exists "$module" || missing+=("$module")
  done
fi
if ((${#missing[@]})); then
  echo "missing dependencies: ${missing[*]}" >&2
  echo 'Arch: sudo pacman -S fcitx5 cmake pkgconf openssl nlohmann-json' >&2
  echo 'Debian / Ubuntu: sudo apt install cmake g++ pkg-config python3 libssl-dev libfcitx5core-dev libfcitx5utils-dev libfcitx5config-dev nlohmann-json3-dev; Rust: https://rustup.rs' >&2
  exit 1
fi
install_prefix=$(python3 -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' "$install_prefix")
cargo_output=$(python3 -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' "${CARGO_TARGET_DIR:-$repo_root/target}")
cargo_args=(build --manifest-path "$repo_root/Cargo.toml" --target-dir "$cargo_output" -p borno-linux-server --locked)
[[ "$build_profile" != release ]] || cargo_args+=(--release)
cargo "${cargo_args[@]}"
cmake_output="$repo_root/target/fcitx5-install-$build_profile"
cmake -S "$repo_root/apps/linux/fcitx5" -B "$cmake_output" "-DCMAKE_BUILD_TYPE=$cmake_type" -DBUILD_TESTING=OFF
cmake --build "$cmake_output" --parallel "${CMAKE_BUILD_PARALLEL_LEVEL:-2}"
python3 "$repo_root/apps/linux/scripts/files.py" install "$install_prefix" "$repo_root" "$cargo_output/$build_profile/borno-linux-server" "$cmake_output/bornopata.so"
printf 'Installed. Start the Server by hand: %s/bin/borno-linux-server\nRestart Fcitx5, then add "Bornopata" in the configuration tool (untick "Only show current language").\n' "$install_prefix"
