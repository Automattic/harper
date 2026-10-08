#!/usr/bin/env python3
"""Give a noun's plural entry the gender of its singular.

*Die Kinder spielt im Garten* is the subject-verb error German teachers mark
most, and `GermanSubjectVerbAgreement` could not see it: nominative *die* is
feminine singular *and* plural, and the noun's recorded number does not break
the tie (see "Why only the plural quantifiers" in README.md). Its gender can.
A noun that is only masculine or neuter cannot stand behind a feminine
singular *die*, so the phrase is a plural.

That needs the plural form to carry a gender, and most do not: `hunde/~~NhE`
has none, and `kinder/~~MhE` carries a masculine flag from the mass import that
wrote `M` on thousands of `-er` words. This script fills the gap where the
evidence is strong:

* the singular `S` has exactly one gender, masculine or neuter, and is a noun
  and nothing else;
* the plural `P` is the form the singular's own plural flag builds (`X` -e,
  `b` -s, `a` -er, `Y` -en), or the umlauted stem plus `-er` or `-e`
  (*Buch*/*Bücher*, *Hof*/*Höfe*);
* `P` has a noun entry of its own with no gender, no other part of speech and
  no feminine derivation.

The dative plural (`-rn`, `-ln`) is skipped: it never follows *die*. So is the
small set in `EXCLUDED`, which are feminine singulars or nouns of their own
that only look like a plural (*Güte* is not the plural of *gut*, *der Taler*
not that of *das Tal*), and the comparative adjectives the mass import
recorded as nouns (`düsterere`).

`kinder/~~MhE` and its siblings get `Z` beside the stray `M` rather than in its
place. Two genders narrow nothing for `GermanDeterminerGender`, which is
right for a plural, and neither is feminine, which is all the subject rule
asks.

Gender flags are plain properties, so this builds no surface form. Idempotent.

    python3 harper-core/src/language/german/scripts/add_german_plural_genders.py
"""

import collections
import pathlib
import re
import sys

DICT = pathlib.Path(__file__).resolve().parent.parent / "dictionary.dict"

LINE = re.compile(r"^(?P<word>[^/\s#]+)/(?P<head>~*)(?P<flags>[^\s#]*)(?P<rest>.*)$")

PLURAL_SUFFIX = {"X": "e", "b": "s", "a": "er", "Y": "en"}
# Part-of-speech flags other than the noun's.
OTHER_POS = set("VJRPOQSTUW")
UMLAUT = {"a": "ä", "o": "ö", "u": "ü"}

EXCLUDED = {"güte", "muse", "premiere", "schläfe", "taler", "fächer"}

# The `-er` plurals of neuter nouns that the import recorded masculine.
MISRECORDED_MASCULINE = """altertümer bretter eier fahrräder felder gelder gesichter güter hühner
kinder kleider lieder nester rathäuser schwerter umfelder ämter bücher häuser bilder dinger
länder bäder gemüter""".split()


# Nouns that exist only in the plural. They get the `+` property (number
# Plural, no gender): *meine Eltern arbeitet* is then a plural subject with a
# singular verb. *Geschwister* and *Möbel* are left out — both have a singular
# in use.
PLURALIA_TANTUM = """Eltern Großeltern Schwiegereltern Stiefeltern Pflegeeltern Leute Ferien Kosten Unkosten
Trümmer Masern Röteln Pocken Alpen Einkünfte Finanzen Gebrüder Personalien Flitterwochen Wirren
Machenschaften Utensilien Annalen Memoiren Textilien Spesen Gliedmaßen Kinkerlitzchen""".split()


def umlauted(stem):
    for i in range(len(stem) - 1, -1, -1):
        if stem[i] in UMLAUT:
            at = i - 1 if stem[i] == "u" and i > 0 and stem[i - 1] == "a" else i
            return stem[:at] + UMLAUT[stem[at]] + stem[at + 1 :]
    return None


def main() -> int:
    lines = DICT.read_text(encoding="utf-8").split("\n")
    flags = collections.defaultdict(str)
    rest = collections.defaultdict(str)
    for line in lines:
        if m := LINE.match(line):
            flags[m["word"].lower()] += m["flags"]
            rest[m["word"].lower()] += m["rest"]

    def genders(word):
        return set("MFZ") & set(flags.get(word, ""))

    plan = {}
    for singular, singular_flags in list(flags.items()):
        recorded = genders(singular)
        if len(recorded) != 1 or "F" in recorded:
            continue
        if "REPLACES" in rest[singular] or OTHER_POS & set(singular_flags):
            continue
        candidates = [singular + PLURAL_SUFFIX[f] for f in PLURAL_SUFFIX if f in singular_flags]
        if stem := umlauted(singular):
            candidates += [stem + "er", stem + "e"]
        for plural in candidates:
            if plural == singular or plural not in flags or plural in EXCLUDED:
                continue
            if plural.endswith(("rn", "ln", "ere")):
                continue
            plural_flags = flags[plural]
            if "N" not in plural_flags or OTHER_POS & set(plural_flags):
                continue
            if "K" in plural_flags or "L" in plural_flags or genders(plural):
                continue
            plan[plural] = next(iter(recorded))
    for plural in MISRECORDED_MASCULINE:
        if genders(plural) == {"M"}:
            plan[plural] = "Z"

    changed = 0
    done = set()
    for i, line in enumerate(lines):
        m = LINE.match(line)
        if not m or m["word"] not in plan or m["word"] in done or "REPLACES" in m["rest"]:
            continue
        gender = plan[m["word"]]
        if gender in m["flags"]:
            continue
        lines[i] = f"{m['word']}/{m['head']}{m['flags']}{gender}{m['rest']}"
        done.add(m["word"])
        changed += 1

    tantum_marked = 0
    tantum_seen = set()
    for i, line in enumerate(lines):
        m = LINE.match(line)
        if not m or m["word"].capitalize() not in PLURALIA_TANTUM or "REPLACES" in m["rest"]:
            continue
        tantum_seen.add(m["word"].capitalize())
        if "+" not in m["flags"]:
            lines[i] = f"{m['word']}/{m['head']}{m['flags']}+{m['rest']}"
            tantum_marked += 1
    missing = [word for word in PLURALIA_TANTUM if word not in tantum_seen]
    if missing:
        while lines and lines[-1] == "":
            lines.pop()
        lines += [f"{word.lower()}/~~Nh+ # plurale tantum, add_german_plural_genders.py" for word in missing]
        lines.append("")
        tantum_marked += len(missing)

    DICT.write_text("\n".join(lines), encoding="utf-8")
    print(f"gave {changed} plural entries a gender, marked {tantum_marked} pluralia tantum", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
