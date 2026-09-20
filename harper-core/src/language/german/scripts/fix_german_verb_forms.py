#!/usr/bin/env python3
"""Restore the conjugated forms German verbs are missing from the word list.

A German verb entry carries one flag per ending it can take: `f` for *ich
lerne*, `i` for *er lernt*, `j` for *wir lernen*, `d` for *er lernte*, `G` for
*du lernst*. An entry that carries some of them but not others has a hole in
its paradigm, and the form simply does not exist for Harper. That is why
`stattfindet`, `denkt` and `verbleibt` were reported as misspellings the moment
the compound decomposition stopped covering for them: `stattfinden/~~hc` has no
`i`, so it cannot form *stattfindet*.

**Which entries count as verbs is not obvious and getting it wrong is the whole
risk.** A filter that trusts `k`, `l`, `e` or `t` proposes `presseinformationt`
and `giftgrünt`, because `k` and `l` are also the n- and en-interfix compound
markers and `e`/`t` sit on noun and adjective entries. Even `V` is unreliable:
`abtragungen`, `sexpuppen` and `landwirtschaftlichen` all carry it. So the flag
filter only narrows the field, and the decision is made by an oracle:

    every form the ending would generate must be a form *of that entry*
    according to `hunspell -m`

That is what keeps `bären -> bärt` and `abtragungen -> abtragungt` out while
letting `ausscheiden -> ausscheidet` and `aufschreiben -> aufschreibt` in. The
`-t` form of a strong verb comes out as the second person plural (*ihr brecht
an*) rather than the third singular with its vowel change (*er bricht an*);
both are German, and the umlauted form is a separate gap this cannot fill.

    scripts/fix_german_verb_forms.py              # report only
    scripts/fix_german_verb_forms.py --apply      # write dictionary.dict

Shares its oracle with `fix_german_noun_forms.py`; see
`german_dictionary_oracle.py`, which also guards against the ISO-8859-1
dictionary in `/usr/share` that silently loses every umlaut.
"""

import collections
import re
import sys

from german_dictionary_oracle import (
    affix_rules,
    analyse,
    apply_additions,
    argument_parser,
    belongs_to,
    check_umlauts_survive,
    entries,
    flagset,
    forms,
)

# The endings this script can restore, in the order they are reported.
CONJUGATION = {
    "f": "ich lerne",
    "i": "er lernt",
    "j": "wir lernen",
    "d": "er lernte",
    "G": "du lernst",
    # The declined present participle. Not in the default `--flags`, because it
    # is an adjective paradigm rather than a conjugation, but it is missing from
    # the same entries and the oracle decides it the same way.
    "c": "der lernende",
}

# Flags that mean "verb" and cannot mean anything else. `V` is included even
# though it is over-applied; the oracle catches what it lets through.
VERB_FLAGS = set("Vjfcd")

# Flags that mean the entry is an inflected noun or adjective rather than an
# infinitive: a plural, or a declined adjective. The bare noun flags are *not*
# here on purpose — every German infinitive can be nominalized (`das Laufen`,
# `das Verbleiben`), so `N` or `Z` on an entry is no argument against it being
# a verb, and excluding them loses `verbleiben` and `überschreiten`.
NOMINAL_FLAGS = set("EXYab") | set("OQSTUW")

INFINITIVE = ("en", "ern", "eln")
# `abzubauen` has no finite forms at all. `zubauen` does, so the `zu` has to be
# preceded by something for the entry to be an extended infinitive.
ZU_INFINITIVE = re.compile(r".+zu[a-zäöüß]+en$")









def candidates(flag, rules, wanted=lambda word: True):
    """Entries that should carry `flag` and do not, with the forms it adds."""
    out = []
    for index, word, flags, comment in entries():
        if not wanted(word):
            continue
        flags_present = flagset(flags)
        if (
            flag in flags_present
            or not flags_present & VERB_FLAGS
            or flags_present & NOMINAL_FLAGS
        ):
            continue
        if not word.endswith(INFINITIVE) or ZU_INFINITIVE.search(word):
            continue
        generated = forms(rules, word)
        if generated:
            out.append((index, word, flags, comment, generated))
    return out


def main():
    # `j` is left out of the default: the form it builds *is* the entry, so
    # the check that the form belongs to the entry proves nothing and `besen`
    # and `boden` come through as verbs.
    parser = argument_parser(__doc__, "fidG")
    args = parser.parse_args()

    check_umlauts_survive(args.hunspell_dict)

    additions = collections.defaultdict(set)
    report = []

    for flag in args.flags:
        rules = affix_rules(flag)
        found = candidates(flag, rules, entry_filter(args.matching))
        stems = analyse(
            (f for _, _, _, _, gen in found for f in gen), args.hunspell_dict
        )

        # Every form the ending builds must be a form of *this* entry. A class
        # that generates two forms and gets one wrong would add the wrong one
        # too, and a form that is some other word entirely is not evidence.
        accepted = [c for c in found if all(belongs_to(c[1], f, stems) for f in c[4])]
        for index, _, _, _, _ in accepted:
            additions[index].add(flag)

        report.append((flag, CONJUGATION[flag], len(found), len(accepted)))
        if accepted:
            sample = ", ".join(f"{w} -> {g[0]}" for _, w, _, _, g in accepted[:4])
            print(f"  {flag} ({CONJUGATION[flag]}): {len(accepted)}/{len(found)}  e.g. {sample}")

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
