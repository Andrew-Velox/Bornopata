#!/usr/bin/env bash
# Builds the release tarball bornopata-<version>-linux-x86_64.tar.gz from already built parts, plus its .sha256 file.
# The parts are built by the release workflow (or by hand: cargo build --release, cmake, borno-gloss-build export).
set -euo pipefail
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
server='' settings='' plugin='' gloss='' dictionary='' model='' licenses='' allow_dev=0 out="$repo_root/target/package"
while (($#)); do
  case "$1" in
    --server) server=${2:?}; shift 2 ;;
    --settings) settings=${2:?}; shift 2 ;;
    --plugin) plugin=${2:?}; shift 2 ;;
    --gloss) gloss=${2:?}; shift 2 ;;
    --dictionary) dictionary=${2:?}; shift 2 ;;
    --lm) model=${2:?}; shift 2 ;;
    --licenses) licenses=${2:?}; shift 2 ;;
    --out) out=${2:?}; shift 2 ;;
    --allow-dev) allow_dev=1; shift ;;
    --help) echo 'usage: package.sh --server FILE --settings FILE --plugin FILE --gloss FILE --dictionary FILE [--lm FILE] [--licenses FILE] [--out DIR] [--allow-dev]'; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
for part in "$server" "$settings" "$plugin" "$gloss" "$dictionary" ${model:+"$model"}; do
  [[ -f "$part" ]] || { echo "missing part: ${part:-<not given>} (see --help)" >&2; exit 2; }
done
version=$("$server" --version | awk '{print $NF}')
[[ -n "$version" && ( "$version" != *-dev* || $allow_dev = 1 ) ]] || { echo "refusing to package a development version: ${version:-unknown} (--allow-dev is for dry runs of the release workflow)" >&2; exit 1; }
name="bornopata-$version-linux-x86_64"
stage="$out/$name"
rm -rf "$stage"
mkdir -p "$stage"/{bin,lib,data,assets/gloss,assets/emoji,assets/dictionary,apps/linux/fcitx5/data,apps/linux/systemd,apps/settings/linux}
install -m 755 "$server" "$stage/bin/borno-linux-server"
install -m 755 "$settings" "$stage/bin/bornopata-settings"
install -m 644 "$plugin" "$stage/lib/bornopata.so"
install -m 644 "$gloss" "$stage/data/gloss-en.bnd"
install -m 644 "$dictionary" "$stage/data/words.bnd"
[[ -z "$model" ]] || install -m 644 "$model" "$stage/data/lm.bnd"
cp "$repo_root/LICENSE" "$repo_root/NOTICE.md" "$stage/"
[[ -z "$licenses" ]] || install -m 644 "$licenses" "$stage/THIRD-PARTY-LICENSES.txt"
cp "$repo_root/assets/gloss/ATTRIBUTION.txt" "$stage/assets/gloss/"
cp "$repo_root/assets/emoji/LICENSE-unicode.txt" "$stage/assets/emoji/"
cp "$repo_root/assets/dictionary/ATTRIBUTION.txt" "$stage/assets/dictionary/"
cp -r "$repo_root/apps/linux/fcitx5/data/addon" "$repo_root/apps/linux/fcitx5/data/inputmethod" "$stage/apps/linux/fcitx5/data/"
cp "$repo_root/apps/linux/systemd/bornopata.service" "$stage/apps/linux/systemd/"
cp "$repo_root/apps/settings/linux/bornopata-settings.desktop" "$stage/apps/settings/linux/"
cp -r "$repo_root/apps/settings/linux/icons" "$stage/apps/settings/linux/"
install -m 755 "$repo_root/apps/linux/scripts/package-install.sh" "$stage/install.sh"
install -m 755 "$repo_root/apps/linux/scripts/uninstall.sh" "$stage/uninstall.sh"
install -m 755 "$repo_root/apps/linux/scripts/files.py" "$stage/files.py"
install -m 755 "$repo_root/apps/linux/scripts/setup.py" "$stage/setup.py"
cat > "$stage/README.txt" <<TEXT
Bornopata $version, a Bangla (Avro phonetic) input method for Fcitx5 on Linux.

Install:    ./install.sh            (into ~/.local; --prefix DIR for another place)
            It also starts the Server, adds Bornopata to Fcitx5 and restarts Fcitx5. Run
            ~/.local/bin/bornopata-setup to do that part again.
Uninstall:  ./uninstall.sh
Settings:   search for "Bornopata Settings" in your application launcher

The licenses and attributions of the program and its data are in this directory and are installed into
share/licenses/bornopata. The program is GPL-3.0-or-later; the dictionary and the glosses are CC BY-SA 4.0.
TEXT
# Reproducible archive: fixed order, owner and time (the commit time when known).
epoch=${SOURCE_DATE_EPOCH:-$(git -C "$repo_root" log -1 --format=%ct 2>/dev/null || date +%s)}
tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$epoch" -C "$out" -czf "$out/$name.tar.gz" "$name"
(cd "$out" && sha256sum "$name.tar.gz" > "$name.tar.gz.sha256")
echo "$out/$name.tar.gz"
