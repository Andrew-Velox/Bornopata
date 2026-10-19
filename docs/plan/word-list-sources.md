# Sources for our own word list

**Status 2026-10-20:** the shipped dictionary is built from the Common Voice Bengali sentence files (CC0) and the Bengali Wikipedia dump (CC BY-SA 4.0). The record with checksums, the resulting license (CC BY-SA 4.0) and the build command is in [assets/dictionary/README.md](../../assets/dictionary/README.md). The other sources below stay unused.

Goal: a Bangla word list with frequencies, whose license lets us redistribute it with a GPL-3.0-or-later project. Licenses were checked on the web on 2026-10-19; **verify the exact license text of a release before relying on it**.

Share-alike note: a word list is a set of facts about a text, and counts are a weak form of derivation, so a frequency list is probably not a copyrightable adaptation. That is an assumption, not settled law. Where a source is share-alike (SA), the safe route is to ship the derived data under the same SA license and credit the source.

| Source | License (as found) | Size and quality | Verdict |
|---|---|---|---|
| Mozilla Common Voice sentences, Bengali (`common-voice/common-voice`, `server/data/bn`: `sentence-collector.txt`, `wiki.bn.txt`) | CC0 (Mozilla's statement; part of it is sentences extracted from Wikipedia, a claim that rests on Mozilla's extraction rules) | 60 MB + 22 MB, 562k sentences, clean | **Used** |
| Bengali Wikipedia dump (`bnwiki-latest-pages-articles.xml.bz2`) | CC BY-SA 4.0 (text also under the GFDL) | 546 MB compressed, about 190k articles, 113M Bengali tokens after cleaning; formal register, template noise (handled by `--wikitext`) | **Used**; derived list carries CC BY-SA 4.0 with attribution |
| Bengali Wiktionary | CC BY-SA 4.0 | small | Optional later |
| IndicCorp v2 (AI4Bharat) | CC0 on the dataset card; one portal lists CC BY-SA 4.0; v1 was CC BY-NC | large news and web text; underlying rights unresolved | Not used |
| Samanantar / BPCC (AI4Bharat) | packaging CC0, seed corpora CC BY 4.0 | parallel sentences | Not used |
| Leipzig Corpora Collection | pages say both CC BY and CC BY-NC | | Not used |
| OSCAR, mC4 | packaging CC0 / ODC-BY; text is Common Crawl, rights unresolved | | Not used |
| Tatoeba (Bengali) | CC BY 2.0 FR | tiny | Test fixtures only, if ever |
| Donations from Bangla publishers or institutions | by agreement | slow, but cleanest for modern vocabulary | Worth asking later |

## Not usable

- Avro's `dictionary.json` and anything we cannot trace: unknown license.
- Published dictionaries (Bangla Academy and similar): copyrighted.
