---
title: Uninstall
order: 3
description: Remove Bornopata with uninstall.sh, which deletes only the files the installer put there and keeps your settings and learned data.
---

1. Stop and disable the Server: `systemctl --user disable --now bornopata`.
2. Run `./uninstall.sh` from the release folder (or `apps/linux/scripts/uninstall.sh` from the source; add `--prefix DIR` if you installed with one).
3. Remove Bornopata from your list in the Fcitx5 configuration tool and restart Fcitx5.

`uninstall.sh` deletes only the files listed in its install manifest, and only if they are unchanged. A file you edited is kept and reported.

**Your data stays.** The config, your phrases, your learned data and the logs are not removed:

```bash
rm -r ~/.config/bornopata ~/.local/share/bornopata ~/.local/state/bornopata
```

removes those too, once you are sure.
