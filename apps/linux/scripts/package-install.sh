#!/usr/bin/env bash
# Installs this prebuilt Bornopata release into a user directory: the Server, the Fcitx5 plugin, the dictionary, the
# glosses and a systemd user unit. Nothing is built and nothing outside the prefix and your XDG directories is touched.
set -euo pipefail
here=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
install_prefix="$HOME/.local"
run_setup=1
while (($#)); do
  case "$1" in
    --prefix) install_prefix=${2:?--prefix needs a path}; shift 2 ;;
    --no-setup) run_setup=0; shift ;;
    --help) echo 'usage: install.sh [--prefix ABSOLUTE_USER_DIR] [--no-setup]'; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ "$install_prefix" = /* && "$install_prefix" != / ]] || { echo 'the install prefix must be an absolute path other than /' >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo 'python3 is needed to run the installer' >&2; exit 1; }
install_prefix=$(python3 -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' "$install_prefix")
model=()
[[ ! -f "$here/data/lm.bnd" ]] || model=(--lm "$here/data/lm.bnd")
python3 "$here/files.py" install "$install_prefix" "$here" "$here/bin/borno-linux-server" "$here/lib/bornopata.so" \
  "$here/data/gloss-en.bnd" "$here/data/words.bnd" --settings "$here/bin/bornopata-settings" "${model[@]}"
printf 'Installed.\nSettings: search for "Bornopata Settings" in your application launcher.\n'
if ((run_setup)); then
  python3 "$here/setup.py"
else
  printf 'Next: %s/bin/bornopata-setup (starts the Server and adds Bornopata to Fcitx5).\n' "$install_prefix"
fi
printf 'If Fcitx5 cannot load the plugin, this release was built against a different Fcitx5: build from source instead (docs/notes/linux-fcitx5.md).\n'
