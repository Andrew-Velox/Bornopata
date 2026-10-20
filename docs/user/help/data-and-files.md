---
title: Your files and your privacy
order: 1
description: Every file Bornopata keeps and where, what is never written or sent, and how to inspect or delete your learned data.
---

## Privacy in short

- Typing, the dictionary, learning, the glosses, emoji and phrases **never use the network**. The Server opens no network connection; it only talks to the plugin and to your display server on your own computer.
- What you type is **never written to a log**. Logs record events such as "session opened" and errors, not text.
- In **password fields** Bornopata does nothing; in fields marked **sensitive** it remembers nothing.
- Learned data stays in plain text files in your home directory, which you can read, edit and delete.

The Server and the plugin talk through a private socket (`$XDG_RUNTIME_DIR/bornopata.sock`) that only your own user can open.

## Files

| File | What it is |
|---|---|
| `~/.config/bornopata/config.toml` | Your settings. See [The config file](../settings/config-file.md). |
| `~/.local/share/bornopata/words.bnd` | The dictionary (CC BY-SA 4.0). |
| `~/.local/share/bornopata/gloss-en.bnd` | The English glosses (CC BY-SA 4.0). |
| `~/.local/share/bornopata/phrases.tsv` | Your own phrases. See [Your own phrases](../input/phrases.md). |
| `~/.local/share/bornopata/user-choices.tsv` | Which word you chose for which letters, with a count and the last day used. |
| `~/.local/share/bornopata/user-words.tsv` | How often you committed each word. |
| `~/.local/share/bornopata/user-ngram.tsv` | Which word you typed after which. |
| `~/.local/share/bornopata/lm.bnd` | An optional language model; not part of this preview. |
| `~/.local/state/bornopata/logs/` | Server logs, one file per day, the last seven kept. |
| `~/.local/lib/fcitx5/bornopata.so`, `~/.local/bin/borno-linux-server` | The plugin and the Server (under the install prefix). |
| `~/.config/systemd/user/bornopata.service` | The unit that starts the Server. |
| `~/.local/share/licenses/bornopata/` | The licences and attributions. |

The three `user-*.tsv` files are the whole of what Bornopata has learned about you. Open them in a text editor to see exactly what is stored. To erase them, stop the Server, delete the files and start it again; see [What it learns](../learning/learning.md).

Installing and uninstalling never touch your config, phrases, learned data or logs.

## Logs

If something goes wrong, the log says why: `~/.local/state/bornopata/logs/server.<date>.log`, or `journalctl --user -u bornopata`. For more detail set `log_level = "debug"` in [the config file](../settings/config-file.md) Debug logs describe what each key did, still without the text you typed.
