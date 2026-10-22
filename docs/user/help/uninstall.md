---
title: Uninstall
order: 3
description: Remove Bornopata with uninstall.sh, which deletes only the files the installer put there and keeps your settings and learned data.
---

Run `./uninstall.sh` from the release folder (or `apps/linux/scripts/uninstall.sh` from the source; add `--prefix DIR` if you installed with one). It stops the Server, takes Bornopata out of your Fcitx5 input methods (restarting Fcitx5) and removes the files. With the AUR package, run `bornopata-setup --remove` first and then `pacman -R bornopata`.

`uninstall.sh` deletes only the files listed in its install manifest, and only if they are unchanged. A file you edited is kept and reported.

**Your data stays.** The config, your phrases, your learned data and the logs are not removed:

```bash
rm -r ~/.config/bornopata ~/.local/share/bornopata ~/.local/state/bornopata
```

removes those too, once you are sure.
