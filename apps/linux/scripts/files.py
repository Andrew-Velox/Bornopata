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
    if len(args) != 3:
        raise SystemExit('install needs: <prefix> <repo root> <server binary> <plugin>')
    root, server, plugin = map(Path, args)
    data = Path(os.environ.get('XDG_DATA_HOME', str(Path.home() / '.local/share')))
    if not data.is_absolute():
        raise SystemExit('XDG_DATA_HOME must be an absolute path')
    files = {prefix / 'share/licenses/bornopata/LICENSE': root / 'LICENSE',
             prefix / 'bin/borno-linux-server': server,
             prefix / 'lib/fcitx5/bornopata.so': plugin}
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
