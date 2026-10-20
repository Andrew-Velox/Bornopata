# Emoji data

The emoji offered beside a word (`hridoy` offers 🫀 ❤️) and for an English word typed in Roman letters (`heart`). `emoji.tsv` is compiled into the Server; `crates/borno-emoji` reads it.

## Committed files

- `emoji.tsv`: 1,573 emoji up to Emoji 15.1, in Unicode's order, one row each: `emoji<TAB>Bangla name<TAB>Bangla keywords<TAB>English name<TAB>English keywords`, keywords separated by `|`. Skin tone variants and components are left out (newer emoji than 15.1 do not draw in common fonts yet).
- `LICENSE-unicode.txt`: the license text the data comes with. The installer puts it in `share/licenses/bornopata/EMOJI-LICENSE-UNICODE.txt`.

## Sources

| Source | License | Used for | Retrieved |
|---|---|---|---|
| [`emoji-test.txt`](https://unicode.org/Public/emoji/latest/emoji-test.txt), Unicode Emoji 18.0 (2026-04-30) | Unicode License v3 | which emoji exist, their sequences (with U+FE0F where needed) and their order | 2026-10-20 |
| [CLDR](https://github.com/unicode-org/cldr) `common/annotations/bn.xml` | Unicode License v3 (`Unicode-3.0`) | the Bangla names and keywords | 2026-10-20 |
| CLDR `common/annotations/en.xml` | Unicode License v3 | the English names and keywords | 2026-10-20 |

The Unicode License v3 is permissive and compatible with GPL-3.0-or-later: the copyright and permission notice has to travel with copies, which `LICENSE-unicode.txt` does.

## Rebuilding

```bash
cargo run -p borno-emoji-build -- --emoji-test emoji-test.txt --bn bn.xml --en en.xml --output assets/emoji/emoji.tsv
```

`--max-version` (default 15.1) leaves out newer emoji.

## How the keywords are used

Only single words are indexed, since one word is typed at a time. A Bangla word the engine offers as a candidate finds the emoji whose Bangla name or keywords it is; an English word typed in lower-case Roman letters finds them by the English ones. The emoji whose own name is the word come first, then the ones with the fewest keywords (the plain red heart before a face with heart eyes), then Unicode's order; at most eight are kept per word. The Bangla keywords are CLDR's own wording and spelling, so a word spelled differently from CLDR (`ভালবাসা` against `ভালোবাসা`) may find a different emoji or none.
