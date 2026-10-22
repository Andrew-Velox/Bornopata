#!/usr/bin/env bash
# Removes only the files in the install manifest. Configuration, dictionaries and learned data stay.
set -euo pipefail
install_prefix="$HOME/.local"
while (($#)); do
  case "$1" in
    --prefix) install_prefix=${2:?--prefix needs a path}; shift 2 ;;
    --help) echo 'usage: uninstall.sh [--prefix ABSOLUTE_DIR_USED_AT_INSTALL]'; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
here=$(dirname -- "${BASH_SOURCE[0]}")
# Stops the Server and takes Bornopata out of Fcitx5's list first; the files are removed afterwards.
python3 "$here/setup.py" --remove
python3 "$here/files.py" uninstall "$install_prefix"
echo 'Uninstalled. User data is kept.'
