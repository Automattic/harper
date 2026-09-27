# Slovak data

`dictionary.dict` and `annotations.json` are generated from
[sk-spell/hunspell-sk](https://github.com/sk-spell/hunspell-sk) by Zdenko Podobný
and contributors. The generator lives in
[sk-spell/harper-sk](https://github.com/sk-spell/harper-sk), which rebuilds both
files whenever hunspell-sk releases, so changes to the words belong in hunspell-sk
and changes to the translation belong there, not here.

Both files are licensed under MPL-2.0, as stated in their headers. MPL-2.0 applies
per file and is compatible with Harper's Apache-2.0.

This is the trimmed build: 58,712 entries expanding to about 608,000 word forms,
which keeps memory at the level of the German dictionary. The full build, about
800,000 entries and 2.9 million forms, is `data/` in the same repository. Entry
selection was filtered using the lemma frequency list of the prim-11.0-public-all
corpus, Slovak National Corpus, Ľ. Štúr Institute of Linguistics, Slovak Academy
of Sciences, v. v. i. <https://korpus.sk>, with the OpenSubtitles frequency list
(hermitdave/FrequencyWords, MIT) as a secondary source. No word from either list
enters the data.
