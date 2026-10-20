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
    gloss = directory / 'gloss-en.bnd'
    server.write_bytes(b'test binary\n')
    plugin.write_bytes(b'test plugin\n')
    gloss.write_bytes(b'test glosses\n')
    server.chmod(0o755)
    config = directory / 'user config'
    env = dict(os.environ, XDG_DATA_HOME=str(data), XDG_CONFIG_HOME=str(config))
    args = ['python3', str(helper), 'install', str(prefix), str(root), str(server), str(plugin), str(gloss)]
    subprocess.run(args, env=env, check=True)
    subprocess.run(args, env=env, check=True)
    addon = data / 'fcitx5/addon/bornopata.conf'
    assert f'Library={prefix}/lib/fcitx5/bornopata\n' in addon.read_text()
    assert (prefix / 'bin/borno-linux-server').stat().st_mode & 0o111
    unit = config / 'systemd/user/bornopata.service'
    assert f'ExecStart={prefix}/bin/borno-linux-server\n' in unit.read_text()
    assert '@PREFIX@' not in unit.read_text()
    # The gloss file goes next to the dictionary, and its attribution into the license directory.
    assert (data / 'bornopata/gloss-en.bnd').read_bytes() == b'test glosses\n'
    assert 'Wiktionary' in (prefix / 'share/licenses/bornopata/GLOSS-ATTRIBUTION.txt').read_text()
    assert 'Unicode-3.0' in (prefix / 'share/licenses/bornopata/EMOJI-LICENSE-UNICODE.txt').read_text()
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
    assert not (data / 'bornopata/gloss-en.bnd').exists()
    assert not (prefix / 'share/licenses/bornopata/GLOSS-ATTRIBUTION.txt').exists()
    assert not (prefix / 'share/licenses/bornopata/EMOJI-LICENSE-UNICODE.txt').exists()
    assert user.read_text() == 'learned data' and other.read_text() == 'another input method'
    assert modified.read_text() == 'edited by the user'

# A packaged release also brings the dictionary and its attribution.
with tempfile.TemporaryDirectory(prefix='borno-install-') as temporary:
    directory = Path(temporary)
    for name in ('server', 'plugin', 'gloss-en.bnd', 'words.bnd', 'settings', 'lm.bnd'):
        (directory / name).write_bytes(f'test {name}\n'.encode())
    env = dict(os.environ, XDG_DATA_HOME=str(directory / 'data'), XDG_CONFIG_HOME=str(directory / 'config'))
    args = ['python3', str(helper), 'install', str(directory / 'prefix'), str(root)]
    args += [str(directory / name) for name in ('server', 'plugin', 'gloss-en.bnd', 'words.bnd')]
    args += ['--settings', str(directory / 'settings'), '--lm', str(directory / 'lm.bnd')]
    subprocess.run(args, env=env, check=True)
    # The settings program comes with a launcher entry that names it by absolute path, and an icon.
    entry = (directory / 'data/applications/bornopata-settings.desktop').read_text()
    assert f"Exec={directory / 'prefix'}/bin/bornopata-settings\n" in entry and '@PREFIX@' not in entry
    assert (directory / 'data/icons/hicolor/scalable/apps/bornopata.svg').is_file()
    assert (directory / 'prefix/bin/bornopata-settings').read_bytes() == b'test settings\n'
    assert (directory / 'data/bornopata/words.bnd').read_bytes() == b'test words.bnd\n'
    assert (directory / 'data/bornopata/lm.bnd').read_bytes() == b'test lm.bnd\n'
    assert 'CC BY-SA 4.0' in (directory / 'prefix/share/licenses/bornopata/DICTIONARY-ATTRIBUTION.txt').read_text()
    subprocess.run(['python3', str(helper), 'uninstall', str(directory / 'prefix')], env=env, check=True)
    assert not (directory / 'data/bornopata/words.bnd').exists() and not (directory / 'data/bornopata/lm.bnd').exists()
    assert not (directory / 'data/applications/bornopata-settings.desktop').exists()
print('install, reinstall, manifest and uninstall tests passed')
