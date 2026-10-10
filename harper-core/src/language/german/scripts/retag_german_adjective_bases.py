#!/usr/bin/env python3
"""Turn adjective base forms that corpus mining filed as nouns into adjectives.

`teuer/~~MhEUW78OQRST` and `abundant/~~NXhOQRST` decline like adjectives and
are adjectives, but they carry a noun property (`N`, `M`, `F`, `Z`, the
singular `A`) and often a noun plural (`X`, `Y`, `E`) on top. Two things go
wrong with that:

* `GermanFalseCapitalization` cannot report *zu Teuer*, because a word with a
  noun reading may always be capitalized.
* the plural affixes stamp the adjective's own declined forms (`abundante`,
  `abundanten`) as plural nouns.

`strip_german_noun_readings.py` leaves every entry with an adjective flag
alone, and rightly so for declined forms: *das Gute* is a noun igerman98 does
not list. The **base form** is different. A nominalized adjective always
carries a declension ending, so a bare `teuer` is never a noun -- unless a noun
of the same letters exists, as *Arm*, *Reich* and *Gut* do.

An entry is retagged when all of these hold:

* hunspell declines it (`SFX A`) as a lower-case headword;
* the capitalized spelling is not in hunspell's expanded form list, so no noun
  *Teuer* exists;
* it carries a noun property or a noun-plural affix, and no adjective
  property `J`.

The noun properties go and `J` comes in. A noun-plural affix goes too, because
each of them is also a noun *property*: the affixes and properties share one
flag namespace. It stays only when it builds a hunspell-known word the
adjective endings `O`/`Q`/`R`/`S`/`T` do not.

    unmunch de_DE.dic de_DE.aff | grep -vE '[|/]' | sort -u > forms.txt
    harper-core/src/language/german/scripts/retag_german_adjective_bases.py --hunspell-dict PATH --forms forms.txt            # report
    harper-core/src/language/german/scripts/retag_german_adjective_bases.py --hunspell-dict PATH --forms forms.txt --apply    # write

PATH is a UTF-8 copy of de_DE without extension; see `german_dictionary_oracle.py`.
"""

import argparse
import pathlib
import sys

from fix_german_adjective_forms import adjective_headwords
from german_dictionary_oracle import DICT, affix_rules, entries, flagset, forms

NOUN_PROPERTIES = set("NMFZA")
NOUN_PLURALS = "XYEab"
DECLENSION = "OQRST"
ADJECTIVE_PROPERTY = "J"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--hunspell-dict", required=True)
    parser.add_argument("--forms", required=True, type=pathlib.Path)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()

    adjectives = {w for w in adjective_headwords(args.hunspell_dict) if w.islower()}
    known_forms = set(args.forms.read_text(encoding="utf-8").split())
    rules = {flag: affix_rules(flag) for flag in NOUN_PLURALS + DECLENSION}

    headwords = {word for _, word, _, _ in entries()}

    changes = {}
    for index, word, flags, _ in entries():
        present = flagset(flags)
        if (
            not word.islower()
            or word not in adjectives
            or word.capitalize() in known_forms
            or ADJECTIVE_PROPERTY in present
            or "^" in present
            or not present & (NOUN_PROPERTIES | set(NOUN_PLURALS))
        ):
            continue

        # A plural affix is also a noun *property* (the flag namespace is
        # shared), so it has to go too -- unless it builds a real word nothing
        # else does. A form hunspell does not know, or one with an entry of its
        # own (`teuern`), is no reason to keep it.
        declined = {f for flag in DECLENSION if flag in present for f in forms(rules[flag], word)}
        dropped = NOUN_PROPERTIES | {
            flag
            for flag in NOUN_PLURALS
            if flag in present
            and all(
                f in declined or f in headwords or f not in known_forms
                for f in forms(rules[flag], word)
            )
        }
        kept = "".join(c for c in flags.lstrip("~") if c not in dropped)
        changes[index] = f"{word}/~~{kept}{ADJECTIVE_PROPERTY}"

    print(f"{len(changes)} adjective base forms carry a noun reading")
    lines = DICT.read_text(encoding="utf-8").splitlines()
    for index in list(changes)[:: max(1, len(changes) // 12)][:12]:
        print(f"    {lines[index]}  ->  {changes[index]}")

    if not args.apply:
        print("dry run; pass --apply to write")
        return 0

    for index, replacement in changes.items():
        lines[index] = f"{replacement} # retag: adjective base, was noun (igerman98 SFX A)"
    DICT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {DICT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
