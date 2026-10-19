#!/usr/bin/env python3
"""Real-file install into a temporary user directory: repeat install, manifest, refusal and uninstall keeping user files."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parents[3]
helper = root / 'apps/linux/scripts/files.py'
with tempfile.TemporaryDirectory(prefix='borno-install-') as temporary:
    directory = Path(temporary)
    prefix = directory / 'prefix with spaces'
    data = directory / 'user data'
    server = directory / 'server'
    plugin = directory / 'plugin'
    server.write_bytes(b'test binary\n')
    plugin.write_bytes(b'test plugin\n')
    server.chmod(0o755)
    config = directory / 'user config'
    env = dict(os.environ, XDG_DATA_HOME=str(data), XDG_CONFIG_HOME=str(config))
    args = ['python3', str(helper), 'install', str(prefix), str(root), str(server), str(plugin)]
    subprocess.run(args, env=env, check=True)
    subprocess.run(args, env=env, check=True)
    addon = data / 'fcitx5/addon/bornopata.conf'
    assert f'Library={prefix}/lib/fcitx5/bornopata\n' in addon.read_text()
    assert (prefix / 'bin/borno-linux-server').stat().st_mode & 0o111
    unit = config / 'systemd/user/bornopata.service'
    assert f'ExecStart={prefix}/bin/borno-linux-server\n' in unit.read_text()
    assert '@PREFIX@' not in unit.read_text()
    manifest = json.loads((prefix / 'share/bornopata/install-manifest.json').read_text())
    assert all(Path(filename).is_file() for filename in manifest)
    user = data / 'bornopata/user.tsv'
    user.parent.mkdir(parents=True, exist_ok=True)
    user.write_text('learned data')
    other = data / 'fcitx5/addon/other.conf'
    other.write_text('another input method')
    modified = unit
    modified.write_text('edited by the user')
    failed = subprocess.run(args, env=env, capture_output=True)
    assert failed.returncode != 0
    subprocess.run(['python3', str(helper), 'uninstall', str(prefix)], env=env, check=True)
    subprocess.run(['python3', str(helper), 'uninstall', str(prefix)], env=env, check=True)
    assert not addon.exists() and not (prefix / 'bin/borno-linux-server').exists()
    assert not (prefix / 'lib/fcitx5/bornopata.so').exists()
    assert user.read_text() == 'learned data' and other.read_text() == 'another input method'
    assert modified.read_text() == 'edited by the user'
print('install, reinstall, manifest and uninstall tests passed')
