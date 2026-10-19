# Draft: question to the upodesh and OpenBangla maintainers

**Status: draft, not sent.** The project owner decides whether and how to send it. Suggested place: an issue on `OpenBangla/upodesh`, cross-linked from `OpenBangla/riti`.

---

Subject: Origin and license of the word list shipped with upodesh and riti

Hello, and thank you for upodesh and riti. We are building a separate Bangla input method (GPL-3.0-or-later) and would like to use upodesh as a dependency.

Before we rely on the bundled word list, could you tell us where it came from and under what license? In particular:

1. `upodesh/data/dictionary.json` and `source-words.txt`: what is the original source of these words, and does it carry its own license?
2. `riti/data/dictionary.json` and `autocorrect.json`: same question. Were the autocorrect pairs written by contributors, or taken from another project?
3. The upodesh README credits `libavrophonetic`. Is the word list derived from the Avro Keyboard dictionary, and if so, under what terms?
4. Is it all right for downstream projects to redistribute the list, and what attribution would you like?

The crates are MIT, so we assume the code is fine to reuse. We only want to be sure about the data, so that we ship something we are allowed to ship.

Thank you for your time.
