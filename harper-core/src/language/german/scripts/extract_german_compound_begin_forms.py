#!/usr/bin/env python3
"""Derive the forms that stand in front of a compound boundary from the Wikidata nouns.

    extract_german_compound_begin_forms.py --lexemes german_lexemes.db

A German compound links its elements with a linking element (*Fugenelement*) that
belongs to the first element: *Arbeit* + *s* + *zeit*, *Festplatte* + *n* +
*speicher*, *Kind* + *er* + *garten*. Harper tries every interfix after every
element, which is how `Arbeitspeicher` and `Versionhinweis` pass.

The lemmas of the Wikidata lexemes (CC0) contain tens of thousands of compound
nouns, and for each of them the split is visible: the head is a noun of its own
and the part in front of it is a noun plus one of the linking elements. This
script collects, for every noun that opens such a compound, the forms it takes
(`arbeit` -> `arbeits`, `kind` -> `kinder`, `festplatte` -> `festplatten`).
`interfix_matches_known_begin` in `spell/compound_checker.rs` reads the list: where
a form is recorded for an element, Harper requires that link; where none is,
Harper keeps guessing.

The output is `compound_begin_forms.txt`, lower case, one form per line.
"""

import argparse
import collections
import pathlib
import sqlite3

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "compound_begin_forms.txt"

LINKS = ("", "s", "n", "en", "er", "es", "e", "ns")
MIN_ELEMENT, MIN_HEAD = 2, 3

HEADER = """\
# Forms that stand in front of a compound boundary, lower case: a noun and the
# linking element it takes (arbeit -> arbeits, kind -> kinder), or a verb stem (mess). Derived from the
# German noun lemmas of the Wikidata lexemes (CC0) by
# harper-core/src/language/german/scripts/extract_german_compound_begin_forms.py.
# Do not edit by hand.
"""


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--lexemes", required=True, help="german_lexemes.db from extract_wikidata_lexemes.py")
    ap.add_argument("--out", default=str(OUT))
    args = ap.parse_args()

    db = sqlite3.connect(f"file:{args.lexemes}?mode=ro", uri=True)
    nouns = {
        lemma
        for (lemma,) in db.execute("select lemma from lexeme where category = 'noun'")
        if lemma.isalpha() and lemma[0].isupper()
    }
    lower = {n.lower() for n in nouns}
    # particles and prepositions that Wikidata also lists as nouns (*Ab*, *An*) take no linking element
    particles = {
        lemma.lower()
        for (lemma,) in db.execute("select lemma from lexeme where category in ('prep', 'adv', 'conj')")
    }
    # the bare stems of the verbs (*mess* for *messen*) and the nouns in *-e* without it (*farb* for *Farbe*)
    stems = set()
    for (lemma,) in db.execute("select lemma from lexeme where category = 'verb'"):
        if lemma.isalpha() and lemma.endswith("n") and len(lemma) > 4:
            stems.add(lemma[:-2] if lemma.endswith("en") and not lemma.endswith(("ern", "eln")) else lemma[:-1])
    stems |= {n[:-1] for n in lower if n.endswith("e") and len(n) >= 5}

    # how often each element links in each way: *Arbeit* links with -s- in hundreds of compounds and with
    # nothing in a handful (*Arbeitnehmer*), and the handful must not license *Arbeitpeicher*
    counts = collections.defaultdict(collections.Counter)  # element -> {form: compounds}
    for noun in nouns:
        word = noun.lower()
        for cut in range(MIN_ELEMENT, len(word) - MIN_HEAD + 1):
            if word[cut:] not in lower:
                continue
            if word[:cut] in stems and word[:cut] not in lower:
                counts[word[:cut]][word[:cut]] += 1  # a verb stem or an -e noun without its e
            for link in LINKS:
                stem_end = cut - len(link)
                if stem_end < MIN_ELEMENT or word[stem_end:cut] != link:
                    continue
                if word[:stem_end] in lower and word[:stem_end] not in particles:
                    counts[word[:stem_end]][word[:cut]] += 1
    forms = set()
    for element, seen in counts.items():
        top = max(seen.values())
        forms |= {form for form, n in seen.items() if n * 10 >= top}
    # *ab* + *s* is not the noun *Abs*: a particle with a link letter on it is no begin form
    forms = {
        f for f in forms
        if not any(f.endswith(link) and f[: -len(link)] in particles for link in LINKS if link)
    }

    pathlib.Path(args.out).write_text(HEADER + "".join(f"{f}\n" for f in sorted(forms)), encoding="utf-8")
    print(f"wrote {len(forms)} forms to {args.out}")


if __name__ == "__main__":
    main()
