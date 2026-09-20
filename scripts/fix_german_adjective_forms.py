#!/usr/bin/env python3
"""Restore the declension German adjectives are missing from the word list.

A German adjective takes five endings — `-e`, `-em`, `-en`, `-er`, `-es` — and
Harper keeps one flag per ending (`O`, `Q`, `R`, `S`, `T`). Two thousand entries
that hunspell declines carry none of them, so only the bare form exists:
`dauerhaft` is a word and `dauerhafter` is not, `fehlerhaft` is and
`fehlerhafter` is not. In running text the declined form is the common one, and
every compound ending in such an adjective fails with it.

**Hunspell decides which entries are adjectives**, because Harper's own flags
cannot: the gap exists precisely where the part of speech was never recorded.
An entry qualifies when hunspell's German dictionary lists the same headword
with `SFX A`, its declension class. Every form the ending would build is then
checked the usual way:

    every generated form must analyse back to this very headword

    dauerhaft -> dauerhafter  st:dauerhaft   -> add the ending
    dauer     -> dauerer      st:dauern      -> leave it alone

That second line is why the hunspell headword list alone is not enough: Harper
and hunspell disagree about what is one entry often enough to matter.

    scripts/fix_german_adjective_forms.py              # report only
    scripts/fix_german_adjective_forms.py --apply      # write dictionary.dict

Shares its oracle with `fix_german_noun_forms.py`; see
`german_dictionary_oracle.py` for the ISO-8859-1 trap it guards against.
"""

import collections
import pathlib
import sys

from german_dictionary_oracle import (
    affix_rules,
    analyse,
    apply_additions,
    argument_parser,
    check_umlauts_survive,
    entries,
    entry_filter,
    flagset,
    forms,
)

# The endings this script can restore, in the order they are reported.
DECLENSION = {
    "O": "das schöne",
    "Q": "dem schönen",
    "R": "die schönen",
    "S": "der schöne",
    "T": "ein schönes",
}

# Hunspell's adjective declension class. An entry it carries is an adjective.
HUNSPELL_ADJECTIVE_FLAG = "A"

# There is deliberately no filter on Harper's own flags. Thousands of these
# adjectives were corpus-mined as nouns and carry a plural (`abschnittsweise`,
# `absatzrelevant`); filtering those out drops the run from 2238 entries to 187,
# which is to say it filters out the problem. The endings are added on top of
# whatever the entry already carries -- taking the wrong noun reading away is a
# separate repair, see scripts/fix_german_pos_flags.py.


def adjective_headwords(dictionary):
    """The lower-cased headwords hunspell declines as adjectives."""
    path = pathlib.Path(f"{dictionary}.dic")
    if not path.exists():
        raise SystemExit(f"no hunspell word list at {path}")
    out = set()
    for line in path.read_text(encoding="utf-8").splitlines()[1:]:
        word, _, flags = line.partition("/")
        if HUNSPELL_ADJECTIVE_FLAG in flags:
            out.add(word.lower())
    return out


def is_form_of(entry, form, stems):
    """Does `form` analyse back to this adjective?

    An adjective is lower case in German, so unlike the noun repair there is no
    capitalization to see through: the stem is the headword itself.
    """
    return any(stem.lower() == entry for stem in stems.get(form, ()))


def candidates(flag, rules, adjectives, wanted):
    """Entries hunspell declines that are missing `flag`, with the forms it adds."""
    out = []
    for index, word, flags, comment in entries():
        if not wanted(word):
            continue
        if flag in flagset(flags):
            continue
        if word.lower() not in adjectives:
            continue
        generated = forms(rules, word)
        if generated:
            out.append((index, word, generated))
    return out


def main():
    parser = argument_parser(__doc__, "OQRST")
    args = parser.parse_args()

    check_umlauts_survive(args.hunspell_dict)
    adjectives = adjective_headwords(args.hunspell_dict)
    print(f"hunspell declines {len(adjectives)} headwords as adjectives")

    additions = collections.defaultdict(set)
    report = []

    for flag in args.flags:
        rules = affix_rules(flag)
        found = candidates(flag, rules, adjectives, entry_filter(args.matching))
        stems = analyse(
            (form for _, _, gen in found for form in gen), args.hunspell_dict
        )
        accepted = [
            c for c in found if all(is_form_of(c[1], form, stems) for form in c[2])
        ]
        for index, _, _ in accepted:
            additions[index].add(flag)

        report.append((flag, DECLENSION[flag], len(found), len(accepted)))
        if accepted:
            sample = ", ".join(f"{w} -> {g[0]}" for _, w, g in accepted[:4])
            print(f"  {flag} ({DECLENSION[flag]}): {len(accepted)}/{len(found)}  e.g. {sample}")

    print(f"\n{'ending':22} {'missing':>9} {'confirmed':>10}")
    for flag, label, found, accepted in report:
        print(f"{flag} ({label}){'':6} {found:9} {accepted:10}")
    print(f"\n{len(additions)} entries to change")

    if not args.apply:
        print("dry run; pass --apply to write")
        return 0

    print(f"wrote {apply_additions(additions)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
