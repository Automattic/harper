#!/usr/bin/env python3
"""Give back the noun reading that shape-based retagging took from real nouns.

Two earlier passes turned noun entries into adjectives by the look of their
ending:

* `622a4b6a6` and its follow-ups (June 2026), marked `#adjective
  (auto-converted)`, 3666 lines;
* `053178de7` (September 2026), marked `retag: adjective (was ~~N…)` for the
  `-al`, `-ent`, `-iv`, `-ar` … shapes, 4119 lines.

Most of them were right: `akzeptabel`, `ambivalent` and `curricular` are
adjectives. But an ending says nothing about the part of speech, and about one
in five was a noun: `Beispiel`, `Februar`, `Mistral`, `Inkrement`,
`Development`, `Dezibel` and the place names in `-tal`. Since then those have had
no noun reading at all, so "is not a noun" cannot be trusted anywhere in the
linters, and *das beispiel* goes unreported.

Hunspell decides, never the shape:

    the capitalized spelling is a hunspell headword        -> a noun exists
    hunspell accepts the lower-case spelling on its own     -> a non-noun exists

The second test asks hunspell to *spell-check* the word (`hunspell -a`) rather
than looking for an `SFX A` headword or analysing it: `hoch` declines
irregularly and is listed only as a compound stem, and `hunspell -m` analyses
the bare compound stem `beispiel` as well. Only the spell check accepts the one
and rejects the other.

    noun only   -> back to a noun; the declension and `J` go
    both        -> `pauschal`/`Pauschal`, `tief`/`Tief`: keep the adjective, add
                   the noun reading and `^`, which says the lower-case spelling
                   is not the noun
    adjective   -> left alone; that retag was right

The noun gets back the flags its comment recorded (`was ~~Nh`), or the bare
`~~Nh` when the comment did not record any. Plurals and genitives are then
restored, verified, by `fix_german_noun_forms.py --matching`, restricted to the
restored words; `--print-matching` prints the expression.

The reverse repair -- adjectives filed as nouns -- is
`retag_german_adjective_bases.py`. The two use the same oracle, so running
them in turn does not undo either one.

    harper-core/src/language/german/scripts/restore_german_shape_retagged_nouns.py --hunspell-dict PATH            # report
    harper-core/src/language/german/scripts/restore_german_shape_retagged_nouns.py --hunspell-dict PATH --apply    # write

PATH is a UTF-8 copy of de_DE without extension (LanguageTool's); see
`german_dictionary_oracle.py`.
"""

import argparse
import collections
import pathlib
import re
import sys

import subprocess

from german_dictionary_oracle import (
    DICT,
    check_umlauts_survive,
    dictionary_encoding,
    entries,
    flagset,
)

MARKERS = ("#adjective (auto-converted)", "retag: adjective (was ~~")
RECORDED = re.compile(r"was ~~([A-Za-z0-9]+)")
ADJECTIVE_FLAGS = set("JOQRSTUWq")


def accepted(words, dictionary):
    """The words `hunspell -a` accepts as spelled."""
    words = sorted(set(words))
    encoding = dictionary_encoding(dictionary)
    result = subprocess.run(
        ["hunspell", "-d", dictionary, "-a"],
        input=("\n".join(words) + "\n").encode(encoding, errors="replace"),
        capture_output=True,
        check=True,
    )
    verdicts = [
        line[:1]
        for line in result.stdout.decode(encoding, errors="replace").splitlines()[1:]
        if line.strip()
    ]
    return {word for word, verdict in zip(words, verdicts) if verdict in "*+-"}


def capitalized_headwords(dictionary):
    """The capitalized headwords of hunspell's word list."""
    lines = pathlib.Path(f"{dictionary}.dic").read_text(encoding="utf-8").splitlines()[1:]
    return {line.partition("/")[0] for line in lines if line[:1].isupper()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--hunspell-dict", required=True, help="UTF-8 de_DE, no extension")
    parser.add_argument("--apply", action="store_true")
    parser.add_argument(
        "--print-matching",
        action="store_true",
        help="print a --matching expression for the nouns this would restore",
    )
    args = parser.parse_args()

    check_umlauts_survive(args.hunspell_dict)
    capitalized = capitalized_headwords(args.hunspell_dict)
    marked = [
        e
        for e in entries()
        if e[1].islower() and any(m.lstrip("#") in e[3] for m in MARKERS)
    ]
    standalone = accepted((e[1] for e in marked), args.hunspell_dict)
    changes = {}
    kinds = collections.Counter()

    for index, word, flags, comment in marked:
        if word.capitalize() not in capitalized:
            continue

        present = flagset(flags)
        if word in standalone:
            if "N" in present:
                continue
            kinds["homograph"] += 1
            changes[index] = (
                f"{word}/{flags}N^ # adjective; the noun is capitalized (hunspell); was retagged by shape"
            )
            continue

        recorded = RECORDED.search(comment)
        restored = recorded.group(1) if recorded else "Nh"
        kept = "".join(
            c for c in flags.replace("~", "") if c not in ADJECTIVE_FLAGS and c not in restored
        )
        kinds["noun"] += 1
        changes[index] = f"{word}/~~{restored}{kept} # noun (hunspell); was retagged by shape"

    if args.print_matching:
        restored = sorted(c.partition("/")[0] for c in changes.values() if "# noun" in c)
        print("^(" + "|".join(restored) + ")$")
        return 0

    print(f"{kinds['noun']} nouns to restore, {kinds['homograph']} homographs to complete")
    lines = DICT.read_text(encoding="utf-8").splitlines()
    for index in list(changes)[:: max(1, len(changes) // 12)][:12]:
        print(f"    {lines[index]}  ->  {changes[index]}")

    if not args.apply:
        print("dry run; pass --apply to write")
        return 0

    for index, replacement in changes.items():
        lines[index] = replacement
    DICT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {DICT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
