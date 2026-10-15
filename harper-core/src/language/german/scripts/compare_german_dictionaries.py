#!/usr/bin/env python3
"""Score an expanded German dictionary against a treebank's gold annotation.

    compare_german_dictionaries.py PROBE.tsv TREEBANK.conllu... [--label NAME]
    compare_german_dictionaries.py --entries dictionary.dict TREEBANK.conllu... [--label NAME]

PROBE.tsv comes from the ignored test `probe_german_dictionary` (see
`tests/dump_dictionary.rs`), run over the treebank's distinct forms: one line
per form, the spelling and the metadata as JSON, empty when Harper's
spell-checking dictionary, compounds included, does not accept it. The score does not read `dictionary.dict`, so two builds of
it can be held against the same yardstick, and it does not read any other
dictionary: the gold is the treebank's own annotation.

With `--entries` the file is a `dictionary.dict` and is expanded here, with
`german_affixes.py`: quick, but without the compound words Harper also accepts,
so only a comparison of two such runs means anything.

Per gold part of speech it reports the share of tokens whose form is a known
word, whose gold part of speech the dictionary also gives that form, and, for
nouns, whose gold gender and number it gives.
"""

import collections
import json
import sys

POS = {
    "NOUN": "noun", "PROPN": "noun", "VERB": "verb", "AUX": "verb",
    "ADJ": "adjective", "ADV": "adverb", "ADP": "preposition",
    "CCONJ": "conjunction", "SCONJ": "conjunction", "DET": "determiner",
    "PRON": "pronoun",
}
GENDER = {"Masc": "MASCULINE", "Fem": "FEMININE", "Neut": "NEUTER"}
NUMBER = {"Sing": "SINGULAR", "Plur": "PLURAL"}


def has(md, key):
    if key == "preposition":
        return bool(md.get("preposition"))
    return md.get(key) is not None


def as_set(v):
    if v is None:
        return set()
    return set(v) if isinstance(v, list) else {v}


def load(dump):
    words = {}
    for line in open(dump, encoding="utf-8"):
        w, js = line.rstrip("\n").split("\t", 1)
        if not js:
            continue
        md = json.loads(js)
        pos = {k for k in POS.values() if has(md, k)}
        noun = (md.get("morphology") or {}).get("noun") or {}
        g, n = as_set(noun.get("gender")), as_set(noun.get("number"))
        key = w
        if key in words:
            o = words[key]
            o[0].update(pos); o[1].update(g); o[2].update(n)
        else:
            words[key] = [pos, g, n]
    return words


def load_entries(path):
    import german_affixes as ga

    aff = ga.Affixes()
    words = {}
    for line in open(path, encoding="utf-8"):
        r = ga.parse_dict_line(line)
        if not r or (r[0].isdigit() and not r[1]):
            continue
        for form, info in aff.expand(*r).items():
            pos = {k for k in POS.values() if k in info.pos}
            key = form.casefold()
            if key in words:
                o = words[key]
                o[0].update(pos); o[1].update(info.gender); o[2].update(info.number)
            else:
                words[key] = [pos, set(info.gender), set(info.number)]
    return words


def feats(s):
    return dict(p.split("=", 1) for p in s.split("|") if "=" in p) if s != "_" else {}


def main(argv):
    label = "dictionary"
    if "--label" in argv:
        i = argv.index("--label"); label = argv[i + 1]; del argv[i:i + 2]
    entries = "--entries" in argv
    if entries:
        argv.remove("--entries")
    words = load_entries(argv[0]) if entries else load(argv[0])
    fold = (lambda w: w.casefold()) if entries else (lambda w: w)
    tot = collections.Counter(); known = collections.Counter(); posok = collections.Counter()
    g_gold = g_ok = g_empty = n_gold = n_ok = n_empty = 0
    types_total = types_known = 0
    seen = set()
    for t in argv[1:]:
        for line in open(t, encoding="utf-8"):
            if line.startswith("#") or not line.strip():
                continue
            c = line.rstrip("\n").split("\t")
            if "-" in c[0] or "." in c[0] or c[3] in ("PUNCT", "NUM", "SYM", "X", "INTJ", "PART"):
                continue
            if not any(ch.isalpha() for ch in c[1]) or any(ch.isdigit() for ch in c[1]) or c[1].endswith("."):
                continue  # numbers, symbols and abbreviations with a stop are not dictionary words
            f, up, ft = fold(c[1]), c[3], feats(c[5])
            tot[up] += 1; tot["ALL"] += 1
            e = words.get(f)
            if f not in seen:
                seen.add(f); types_total += 1; types_known += e is not None
            if e is None:
                continue
            known[up] += 1; known["ALL"] += 1
            if up in POS and POS[up] in e[0]:
                posok[up] += 1; posok["ALL"] += 1
            if up == "NOUN":
                if "Gender" in ft and ft["Gender"] in GENDER:
                    g_gold += 1
                    if not e[1]: g_empty += 1
                    elif GENDER[ft["Gender"]] in e[1]: g_ok += 1
                if "Number" in ft and ft["Number"] in NUMBER:
                    n_gold += 1
                    if not e[2]: n_empty += 1
                    elif NUMBER[ft["Number"]] in e[2]: n_ok += 1
    print(f"== {label}: {len(words)} distinct forms")
    print(f"types known: {types_known}/{types_total} = {100*types_known/types_total:.2f}%")
    print(f"{'UPOS':6s} {'tokens':>8s} {'known%':>7s} {'POS-ok%':>8s}")
    for up in ["ALL", "NOUN", "PROPN", "VERB", "AUX", "ADJ", "ADV", "ADP", "DET", "PRON", "CCONJ", "SCONJ"]:
        if tot[up]:
            print(f"{up:6s} {tot[up]:8d} {100*known[up]/tot[up]:7.2f} {100*posok[up]/tot[up]:8.2f}")
    print(f"noun gender: gold {g_gold}  right {100*g_ok/g_gold:.2f}%  unknown {100*g_empty/g_gold:.2f}%  wrong {100*(g_gold-g_ok-g_empty)/g_gold:.2f}%")
    print(f"noun number: gold {n_gold}  right {100*n_ok/n_gold:.2f}%  unknown {100*n_empty/n_gold:.2f}%  wrong {100*(n_gold-n_ok-n_empty)/n_gold:.2f}%")


if __name__ == "__main__":
    main(sys.argv[1:])
