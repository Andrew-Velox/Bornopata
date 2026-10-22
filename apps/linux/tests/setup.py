#!/usr/bin/env python3
"""The profile edits of bornopata-setup, on Fcitx5's real profile layout, and the whole script in a scratch home."""
import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parents[3]
script = root / 'apps/linux/scripts/setup.py'
spec = importlib.util.spec_from_file_location('bornopata_setup', script)
setup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(setup)

DEFAULT = """[Groups/0]
# Group Name
Name=Default
# Layout
Default Layout=us
# Default Input Method
DefaultIM=keyboard-us

[Groups/0/Items/0]
# Name
Name=keyboard-us
# Layout
Layout=

[GroupOrder]
0=Default
"""

added = setup.add_to_profile(DEFAULT)
assert '[Groups/0/Items/1]\n# Name\nName=bornopata' in added
assert added.index('Items/1') < added.index('[GroupOrder]')
assert '\n\n[Groups/0/Items/1]' in added and 'Layout=\n\n[GroupOrder]' in added
assert setup.add_to_profile(added) == added, 'adding twice changes nothing'
assert setup.remove_from_profile(added).replace('\n\n\n', '\n\n').strip() == DEFAULT.strip()

# Two groups: only the first one gets it, and the numbering continues.
two = DEFAULT.replace('[GroupOrder]', '[Groups/1]\nName=Other\n\n[Groups/1/Items/0]\nName=keyboard-de\n\n[GroupOrder]')
result = setup.add_to_profile(two)
assert result.index('Name=bornopata') < result.index('[Groups/1]')
assert setup.add_to_profile('[GroupOrder]\n0=Default\n') is None, 'no first group, nothing to edit'

# The whole script never runs with the Fcitx5 step here: that would quit the Fcitx5 of whoever runs the tests.
with tempfile.TemporaryDirectory(prefix='borno-setup-') as scratch:
    home = Path(scratch)
    profile = home / 'fcitx5/profile'
    profile.parent.mkdir()
    profile.write_text(DEFAULT)
    env = dict(os.environ, XDG_CONFIG_HOME=str(home))
    subprocess.run(['python3', str(script), '--no-service', '--no-fcitx'], env=env, check=True, capture_output=True)
    subprocess.run(['python3', str(script), '--remove', '--no-service', '--no-fcitx'], env=env, check=True, capture_output=True)
    assert profile.read_text() == DEFAULT
    wrong = subprocess.run(['python3', str(script), '--nope'], env=env, capture_output=True)
    assert wrong.returncode != 0

    # Writing keeps one backup of the original, and the edit survives a round trip through the file.
    setup_path = setup.profile_path
    os.environ['XDG_CONFIG_HOME'] = str(home)
    setup.write_profile(profile, added)
    setup.write_profile(profile, setup.remove_from_profile(added))
    backup = profile.with_name('profile.bornopata-backup')
    assert backup.read_text() == DEFAULT
    assert setup_path() == profile
print('setup script tests passed')
