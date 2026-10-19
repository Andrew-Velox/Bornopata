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
python3 "$(dirname -- "${BASH_SOURCE[0]}")/files.py" uninstall "$install_prefix"
echo 'Uninstalled. Run: systemctl --user disable --now bornopata (if you enabled it), then restart Fcitx5. User data is kept.'
