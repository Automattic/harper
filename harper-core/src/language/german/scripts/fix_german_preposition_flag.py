#!/usr/bin/env python3
"""Take the preposition flag off the entries that are nouns.

`P` is the preposition property, and 300 entries carry it. Two thirds of them
are not prepositions at all but ordinary nouns that a bulk import stamped with
the wrong letter: `augur/~~P`, `abbreviatur/~~P`, `abduktion/~~P`,
`bodendenkmal/~~P`, `buhlschaft/~~P`.

The damage is not cosmetic. Such an entry has no noun property, so:

  * the plural and genitive repairs skip it — they look for a noun flag first,
    which is why `Adaptationen` and `Appositionen` could never be restored;
  * `GermanNounCapitalization` sees no noun reading;
  * `short_element_usable` lets a two-letter entry into a compound when it is a
    noun *or a preposition*, so the wrong letter widens that gate too.

The oracle is `hunspell -m` on the capitalized spelling, the same test the other
noun repairs use: German capitalizes its nouns and so does hunspell's
dictionary.

    Abduktion   st:Abduktion    a noun       -> P becomes N
    An          st:an           a preposition -> left alone

The entries hunspell does not know at all are left alone too. Several are
import typos (`brandwein`, `besenstil`, `bindestich`) and deleting or retagging
them is a separate question.

    scripts/fix_german_preposition_flag.py              # report only
    scripts/fix_german_preposition_flag.py --apply      # write dictionary.dict
"""

import sys

from german_dictionary_oracle import (
    DICT,
    analyse,
    argument_parser,
    check_umlauts_survive,
    entries,
    flagset,
)

PREPOSITION_FLAG = "P"
NOUN_FLAG = "N"


def capitalized(word):
    return word[:1].upper() + word[1:]


def is_a_noun(word, stems):
    """Does hunspell report a capitalized stem for the capitalized spelling?"""
    return any(stem[:1].isupper() for stem in stems.get(capitalized(word), ()))


def rewrite(flags):
    """Replace `P` with `N`, unless the entry already has a noun property."""
    letters = flagset(flags)
    replacement = "" if letters & set("NMFZz") else NOUN_FLAG
    return flags.replace(PREPOSITION_FLAG, replacement, 1)


def main():
    parser = argument_parser(__doc__, "")
    args = parser.parse_args()
    check_umlauts_survive(args.hunspell_dict)

    found = [
        (index, word, flags)
        for index, word, flags, _ in entries()
        if PREPOSITION_FLAG in flagset(flags)
    ]
    print(f"{len(found)} entries carry the preposition flag")

    stems = analyse(
        [capitalized(word) for _, word, _ in found], args.hunspell_dict
    )
    changes = {
        index: (word, flags, rewrite(flags))
        for index, word, flags in found
        if is_a_noun(word, stems)
    }
    print(f"{len(changes)} of them are nouns hunspell knows")
    for index in sorted(changes)[:15]:
        word, old, new = changes[index]
        print(f"    {word}/{old}  ->  {word}/{new}")

    if not args.apply:
        print("\nnothing written; pass --apply")
        return 0

    lines = DICT.read_text(encoding="utf-8").splitlines()
    for index, (word, _, new) in changes.items():
        _, sep, comment = lines[index].partition("#")
        lines[index] = f"{word}/{new}" + (f" {sep}{comment}" if sep else "")
    DICT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"\nwrote {DICT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
