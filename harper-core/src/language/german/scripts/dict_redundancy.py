#!/usr/bin/env python3
"""Where does the German dictionary's size come from? Classify its headwords.

Reads only `dictionary.dict`. The classes are heuristics, meant to point at
what to look at, not to decide what to delete:

  name       carries the proper-name flag `!`
  compound   splits into two other headwords (optionally joined by a linking
             element: Sicherheit+s+Organe); the compound checker may already
             accept it without the entry
  form       a suffix away from another headword of the same spelling stem
             (Betretungsverbote vs Betretungsverbot, anwarb vs anwarben);
             some are kept on purpose, e.g. the plural marker `&`
  other      everything else

With --write-split FILE it writes the dictionary without the `compound` class,
so that a build with and without them can be compared (see
`just language-perf`, and the spell check over the removed words).

Usage: dict_redundancy.py [--examples N] [--write-split FILE]
"""
import argparse
import collections
import random
import re
from pathlib import Path

DICT = Path(__file__).resolve().parent.parent / "dictionary.dict"
LINKS = ["", "s", "es", "n", "en", "e", "er", "ns", "ens"]
FORM_ENDINGS = ["e", "en", "n", "s", "es", "er", "em", "et", "est", "st", "t", "te", "ten", "tet", "test", "ete", "eten"]
MIN_HEAD, MIN_MOD = 4, 3


def load():
    entries = []
    for line in DICT.read_text(encoding="utf8").splitlines():
        if not line or line.startswith("#"):
            continue
        word, _, rest = line.partition("/")
        entries.append((word, rest.split("#")[0].strip().lstrip("~")))
    return entries


def split_compound(word, known):
    """A (modifier, link, head) split whose parts are both known words, or None."""
    low = word.lower()
    for i in range(MIN_MOD, len(low) - MIN_HEAD + 1):
        head, front = low[i:], low[:i]
        if head not in known:
            continue
        for link in LINKS:
            if link and not front.endswith(link):
                continue
            mod = front[: len(front) - len(link)] if link else front
            if len(mod) >= MIN_MOD and mod in known:
                return mod, link, head
    return None


def is_form(word, known):
    low = word.lower()
    for end in FORM_ENDINGS:
        if low.endswith(end) and len(low) - len(end) >= 4 and low[: -len(end)] in known:
            return low[: -len(end)]
    return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--examples", type=int, default=8)
    ap.add_argument("--write-split", type=Path)
    a = ap.parse_args()
    entries = load()
    names = {w for w, f in entries if "!" in f}
    known = {w.lower() for w, f in entries if "!" not in f}
    classes = collections.defaultdict(list)
    for w, f in entries:
        if w in names:
            classes["name"].append(w)
        elif (s := split_compound(w, known)):
            classes["compound"].append(w)
        elif (b := is_form(w, known)):
            classes["form"].append(w)
        else:
            classes["other"].append(w)
    n = len(entries)
    print(f"{n} headwords")
    random.seed(0)
    for k in ["name", "compound", "form", "other"]:
        ws = classes[k]
        ex = ", ".join(random.sample(ws, min(a.examples, len(ws))))
        print(f"{k:9}{len(ws):8}  {100*len(ws)/n:5.1f}%   e.g. {ex}")
    if a.write_split:
        drop = set(classes["compound"])
        out = [l for l in DICT.read_text(encoding="utf8").splitlines() if l.partition("/")[0] not in drop]
        a.write_split.write_text("\n".join(out) + "\n", encoding="utf8")
        print(f"wrote {a.write_split} without {len(drop)} compounds")


if __name__ == "__main__":
    main()
