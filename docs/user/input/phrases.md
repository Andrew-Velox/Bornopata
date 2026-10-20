---
title: Your own phrases
order: 3
description: Give any text a short Roman code, such as mymail for your email address, and it comes first in the list when you type the code.
---

A phrase is a short code that types a longer text: your email address, a signature, a greeting, a line you write every day. It can be in any language.

## Add a phrase

Open `~/.local/share/bornopata/phrases.tsv` in a text editor. Bornopata creates the file the first time the Server starts, with instructions at the top. Add one phrase per line: the **code**, a **Tab**, then the **text**.

```text
mymail	me@example.com
sig	ধন্যবাদ,\nমোহাব্বত
ty	অনেক ধন্যবাদ!
```

Save the file. Within a couple of seconds Bornopata picks the change up; there is nothing to restart.

Now typing `mymail` puts `me@example.com` first in the candidate list, and `Space` types it.

## The rules

- A **code** is Roman letters only (`a`–`z`, `A`–`Z`) and is matched **exactly as you type it**, capitals included. Only an exact match counts: `mymai` does not offer `mymail`'s text.
- In the **text**, `\n` is a line break, `\t` a tab and `\\` a backslash.
- Several lines with the same code give several candidates, in the order you list them.
- Lines that start with `#` and blank lines are ignored.
- The ordinary words for the same letters stay in the list, behind your phrases.
- A line that cannot be used (no Tab, a code with digits, empty text) is skipped. The Server writes a warning to its log with the line number, and the other lines still work.

## Privacy

Phrases are **never learned**. Typing a phrase does not add it to what Bornopata remembers about you, so an address or a signature stays only in `phrases.tsv`. See [Your files and your privacy](../help/data-and-files.md).
