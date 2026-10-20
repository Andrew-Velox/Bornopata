#!/usr/bin/env python3
"""Install manifest and checks. Does not manage desktop services, system files or user data."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(chunk)
    return value.hexdigest()


def main():
    action, prefix_text, *args = sys.argv[1:]
    prefix = Path(prefix_text)
    if not prefix.is_absolute() or prefix == Path('/'):
        raise SystemExit('the install prefix must be an absolute path other than /')
    prefix = prefix.resolve()
    manifest = prefix / 'share/bornopata/install-manifest.json'
    old = json.loads(manifest.read_text()) if manifest.is_file() else {}
    if action == 'uninstall':
        for filename, checksum in old.items():
            path = Path(filename)
            if path.is_file() and not path.is_symlink() and digest(path) == checksum:
                path.unlink()
            elif path.exists():
                print(f'keeping modified file: {path}')
        manifest.unlink(missing_ok=True)
        return
    # The settings program is optional: --settings FILE adds it, with its launcher entry and icon.
    settings = None
    if '--settings' in args:
        at = args.index('--settings')
        settings = Path(args[at + 1])
        del args[at:at + 2]
    # The language model of word pairs is optional too: --lm FILE.
    model = None
    if '--lm' in args:
        at = args.index('--lm')
        model = Path(args[at + 1])
        del args[at:at + 2]
    if len(args) not in (4, 5):
        raise SystemExit('install needs: <prefix> <repo root> <server binary> <plugin> <gloss file> [dictionary file] [--settings FILE] [--lm FILE]')
    root, server, plugin, gloss, *dictionary = map(Path, args)
    data = Path(os.environ.get('XDG_DATA_HOME', str(Path.home() / '.local/share')))
    if not data.is_absolute():
        raise SystemExit('XDG_DATA_HOME must be an absolute path')
    files = {prefix / 'share/licenses/bornopata/LICENSE': root / 'LICENSE',
             prefix / 'bin/borno-linux-server': server,
             prefix / 'lib/fcitx5/bornopata.so': plugin}
    files[prefix / 'share/licenses/bornopata/GLOSS-ATTRIBUTION.txt'] = root / 'assets/gloss/ATTRIBUTION.txt'
    files[prefix / 'share/licenses/bornopata/EMOJI-LICENSE-UNICODE.txt'] = root / 'assets/emoji/LICENSE-unicode.txt'
    files[data / 'bornopata/gloss-en.bnd'] = gloss
    if dictionary:
        # A packaged release brings the dictionary; installing from source leaves it to the user (docs/notes/linux-fcitx5.md).
        files[prefix / 'share/licenses/bornopata/DICTIONARY-ATTRIBUTION.txt'] = root / 'assets/dictionary/ATTRIBUTION.txt'
        files[data / 'bornopata/words.bnd'] = dictionary[0]
    if model:
        files[data / 'bornopata/lm.bnd'] = model
    desktop = data / 'applications/bornopata-settings.desktop'
    if settings:
        files[prefix / 'bin/bornopata-settings'] = settings
        files[desktop] = root / 'apps/settings/linux/bornopata-settings.desktop'
        files[data / 'icons/hicolor/scalable/apps/bornopata.svg'] = root / 'apps/settings/linux/bornopata.svg'
    for kind in ('addon', 'inputmethod'):
        files[data / f'fcitx5/{kind}/bornopata.conf'] = root / f'apps/linux/fcitx5/data/{kind}/bornopata.conf'
    config = Path(os.environ.get('XDG_CONFIG_HOME', str(Path.home() / '.config')))
    if not config.is_absolute():
        raise SystemExit('XDG_CONFIG_HOME must be an absolute path')
    unit = config / 'systemd/user/bornopata.service'
    files[unit] = root / 'apps/linux/systemd/bornopata.service'
    # Check every target first so files from another source are never overwritten.
    for target in files:
        if target.is_symlink() or (target.exists() and (str(target) not in old or digest(target) != old[str(target)])):
            raise SystemExit(f'target exists and is not part of this install: {target}')
    installed = {}
    for target, source in files.items():
        target.parent.mkdir(parents=True, exist_ok=True)
        temporary = target.with_name(target.name + '.borno-tmp')
        shutil.copy2(source, temporary)
        if target == data / 'fcitx5/addon/bornopata.conf':
            temporary.write_text(temporary.read_text().replace('Library=bornopata\n', f'Library={prefix}/lib/fcitx5/bornopata\n'))
        if target == desktop:
            temporary.write_text(temporary.read_text().replace('@PREFIX@', str(prefix)))
        if target == unit:
            temporary.write_text(temporary.read_text().replace('@PREFIX@', str(prefix)))
        temporary.replace(target)
        installed[str(target)] = digest(target)
    for filename, checksum in old.items():
        path = Path(filename)
        if filename not in installed and path.is_file() and not path.is_symlink() and digest(path) == checksum:
            path.unlink()
    manifest.parent.mkdir(parents=True, exist_ok=True)
    manifest.write_text(json.dumps(installed, ensure_ascii=False, indent=2) + '\n')


if __name__ == '__main__':
    main()
