---
title: The dictionary
order: 4
description: The word list behind the candidates, about 413,000 Bangla words with frequencies, built from Bengali Wikipedia and Common Voice, under CC BY-SA 4.0.
---

Bornopata's candidates come from a dictionary of about **413,000 Bangla words** with how often each occurs. It is the file `~/.local/share/bornopata/words.bnd`, loaded in a few milliseconds when the Server starts.

## Where it comes from

The word counts were made from **Bengali Wikipedia** (CC BY-SA 4.0) and the **Mozilla Common Voice** Bengali sentences (CC0). Only words and how often they occur are kept, never sentences. The dictionary is distributed under **CC BY-SA 4.0**, with the attribution in `share/licenses/bornopata/DICTIONARY-ATTRIBUTION.txt`.

On the evaluation set of 142 common words with the spellings people type for them, the right word is first in about 92% of cases and within the first nine in about 96%. The set is small and the ranking was tuned on it, so treat that as a rough guide.

## The fallback list

Where the dictionary has no word spelled like your input, a small built-in word list (from the `upodesh` library, MIT) fills in. The Avro conversion of your letters is always offered too, so you can always type a word that is in no list.

## Using another dictionary

Set `BORNOPATA_DICT` to the path of a `.bnd` file before the Server starts to use it instead of `words.bnd`. Building your own is a developer task (`tools/dict-build`).

## Your own words

You do not need to edit the dictionary to add words: just type them. A word that is in no dictionary is offered as the Avro conversion, and once you choose it, [it is learned](../learning/learning.md). For longer texts use [phrases](../input/phrases.md).
