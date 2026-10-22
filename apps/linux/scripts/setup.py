#!/usr/bin/env python3
"""Sets Bornopata up for the current user once its files are installed: starts the Server, adds Bornopata to the
input methods of Fcitx5 and restarts Fcitx5, so nothing is left to do by hand. `--remove` undoes it."""
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

NAME = 'bornopata'
ITEM = re.compile(r'^\[Groups/0/Items/(\d+)\]\s*$')


def profile_path():
    config = Path(os.environ.get('XDG_CONFIG_HOME') or Path.home() / '.config')
    return config / 'fcitx5/profile'


def group_zero_items(lines):
    """(index, first line, end line) of every input method of the first group; the end is exclusive."""
    items = []
    for number, line in enumerate(lines):
        match = ITEM.match(line)
        if match:
            end = next((n for n in range(number + 1, len(lines)) if lines[n].startswith('[')), len(lines))
            items.append((int(match.group(1)), number, end))
    return items


def has_item(lines, start, end):
    return any(line.strip() == f'Name={NAME}' for line in lines[start:end])


def add_to_profile(text):
    """The profile with Bornopata added to its first group. `text` unchanged when it is there already, `None` when
    the profile has no first group to add it to (Fcitx5 has not written one yet)."""
    lines = text.splitlines()
    if not any(line.strip() == '[Groups/0]' for line in lines):
        return None
    items = group_zero_items(lines)
    if any(has_item(lines, start, end) for _, start, end in items):
        return text
    index = max((number for number, _, _ in items), default=-1) + 1
    block = [f'[Groups/0/Items/{index}]', '# Name', f'Name={NAME}', '# Layout', 'Layout=']
    if items:
        end = max(end for _, _, end in items)
        # Keep the blank line that separates sections after the new one.
        while end > 0 and not lines[end - 1].strip():
            end -= 1
        lines[end:end] = ['', *block]
    else:
        at = next(number for number, line in enumerate(lines) if line.strip() == '[Groups/0]')
        end = next((n for n in range(at + 1, len(lines)) if lines[n].startswith('[')), len(lines))
        lines[end:end] = [*block, '']
    return '\n'.join(lines) + '\n'


def remove_from_profile(text):
    """The profile without Bornopata."""
    lines = text.splitlines()
    for _, start, end in reversed(group_zero_items(lines)):
        if has_item(lines, start, end):
            del lines[start:end]
    return '\n'.join(lines) + '\n'


def write_profile(path, text):
    backup = path.with_name(path.name + '.bornopata-backup')
    if not backup.exists():
        shutil.copy2(path, backup)
    temporary = path.with_name(path.name + '.borno-tmp')
    temporary.write_text(text)
    temporary.replace(path)


def fcitx_pids():
    uid = os.getuid()
    pids = []
    for entry in Path('/proc').iterdir():
        if not entry.name.isdigit():
            continue
        try:
            if (entry / 'comm').read_text().strip() == 'fcitx5' and entry.stat().st_uid == uid:
                pids.append(int(entry.name))
        except OSError:
            continue
    return pids


def stop_fcitx():
    """Quits Fcitx5 and waits for it, so it cannot write its old list of input methods over the new one."""
    if not fcitx_pids():
        return False
    if shutil.which('fcitx5-remote'):
        subprocess.run(['fcitx5-remote', '-e'], check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    for _ in range(40):
        if not fcitx_pids():
            return True
        time.sleep(0.1)
    return not fcitx_pids()


def start_fcitx():
    if not (os.environ.get('WAYLAND_DISPLAY') or os.environ.get('DISPLAY')):
        return False
    subprocess.Popen(['fcitx5', '-d'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
    return True


def systemctl(*arguments):
    if not shutil.which('systemctl'):
        return False
    result = subprocess.run(['systemctl', '--user', *arguments], check=False, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    if result.returncode != 0 and result.stderr.strip():
        print(f'  systemctl --user {" ".join(arguments)}: {result.stderr.strip()}')
    return result.returncode == 0


def set_up(service, fcitx):
    if service:
        # A restart, not only a start: after an update the running Server is the old one.
        ok = systemctl('daemon-reload') and systemctl('enable', NAME) and systemctl('restart', NAME)
        print('Server: running, and it starts at every login.' if ok else
              'Server: could not be started as a service. Run borno-linux-server yourself, or fix systemd --user.')
    if not fcitx:
        return
    if not shutil.which('fcitx5'):
        print('Fcitx5: not found. Install Fcitx5, then run bornopata-setup again.')
        return
    path = profile_path()
    stopped = stop_fcitx()
    if path.is_file():
        updated = add_to_profile(path.read_text())
        if updated is None:
            print('Fcitx5: its list of input methods is not set up yet. Open fcitx5-configtool once, then run bornopata-setup again.')
        elif updated != path.read_text():
            write_profile(path, updated)
            print('Fcitx5: Bornopata added to your input methods.')
        else:
            print('Fcitx5: Bornopata is in your input methods already.')
    else:
        print('Fcitx5: no list of input methods yet. Open fcitx5-configtool once, add Bornopata, or run bornopata-setup again.')
    if start_fcitx():
        print('Fcitx5: restarted.' if stopped else 'Fcitx5: started.')
    else:
        print('Fcitx5: not started (no graphical session here). It picks Bornopata up the next time it starts.')
    print('Switch to Bornopata with Fcitx5\'s switch key (often Ctrl+Space) and type: ami')


def remove(service, fcitx):
    if service:
        systemctl('disable', '--now', NAME)
        print('Server: stopped and no longer started at login.')
    path = profile_path()
    if fcitx and path.is_file() and f'Name={NAME}' in path.read_text():
        stop_fcitx()
        write_profile(path, remove_from_profile(path.read_text()))
        start_fcitx()
        print('Fcitx5: Bornopata removed from your input methods.')


def main(arguments):
    if '--help' in arguments:
        print('usage: bornopata-setup [--remove] [--no-service] [--no-fcitx]')
        return
    known = {'--remove', '--no-service', '--no-fcitx'}
    unknown = [argument for argument in arguments if argument not in known]
    if unknown:
        raise SystemExit(f'unknown argument: {unknown[0]}')
    action = remove if '--remove' in arguments else set_up
    action('--no-service' not in arguments, '--no-fcitx' not in arguments)


if __name__ == '__main__':
    main(sys.argv[1:])
