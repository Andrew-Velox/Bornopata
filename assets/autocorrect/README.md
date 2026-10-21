# Autocorrect data

The Bangla spelling of English words typed as they are written, so that `facebook` gives ফেসবুক where the Avro rules would give ফাচেবুক. `autocorrect.tsv` is compiled into the Server; `crates/borno-autocorrect` reads it.

## Committed files

- `autocorrect.tsv`: one row per spelling, `word<TAB>Bangla`, the word in lower-case Roman letters. A word with several rows gives several candidates, in file order. About 360 words: internet and phones, school, health, work and money, travel, everyday talk, sport and food, things at home, news, months and places.

## Source and license

Written for this project from general knowledge of how these words are spelled in Bangla newspapers and everyday writing. No third-party list was copied: not OpenBangla's autocorrect file, not Avro Keyboard's, and not any dictionary. The words and their spellings are facts of the language, so the file is released under CC0-1.0 and ships with the program under GPL-3.0-or-later like the rest of the repository.

The spellings have not yet been checked by a second Bangla reader. Loanword spelling varies (ডেটা and ডাটা, পিৎজা and পিজা); where it does, the file lists the common ones, the more usual first.

## Choosing words

A word is left out when typing it the Avro way gives a common Bangla word, because the correction would take it from someone writing that word: `math` (মাঠ), `call` (চাল), `cup` (চুপ), `cool` (চুল), `cola` (চলা), `mall` (মাল), `mail` (মাইল), `data` (দাতা). To test a new word, look at what the Avro conversion and the dictionary give for it:

```bash
cargo run -p borno-cli -- --dict ~/.local/share/bornopata/words.bnd <word>
```

If a real, frequent word comes first, do not add it. A user who wants such a word anyway can put it in `phrases.tsv`.

## How a typed word finds its row

The typed letters match a row when they are all lower case (`facebook`), start with one capital (`Facebook`) or are all capitals (`TV`). Any other mix of capitals is Avro spelling and is left alone. After a word, one Bangla ending may follow, written as Avro writes it: `e`, `er`, `ke`, `te`, `ra`, `der`, `gulo`, `guli`, `ta`, `ti`, `r`, `o`, `i`. So `facebooke` gives ফেসবুকে and `tvte` gives টিভিতে. An ending needs a word of at least three letters before it (two for an abbreviation without a vowel, like `tv`), and `e` and `er` are not added to a word that ends in a vowel sign.
