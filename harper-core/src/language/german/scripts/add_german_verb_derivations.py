#!/usr/bin/env python3
"""Restore the words German verbs derive, and the word list is missing.

`Verschiebung`, `automatisierter` and `beeinflussbare` are ordinary German
words, and none of them was in Harper's dictionary. Every compound with such a
word at the end then fails too: `Achslastverschiebung`,
`Bedeutungsverschiebungen` and `Blattverschiebung` were all reported as
misspellings.

It handles three suffixes, all of which build a *new word* out of a verb rather
than another form of it:

    7, 8   the '-ung' noun and its plural   verschieb  -> Verschiebung(en)
    n      the declined past participle     automatisier -> automatisierter
    x      the '-bar' adjective             beeinfluss -> beeinflussbare

**Why this needs its own script.** `fix_german_verb_forms.py` only looks at
entries that end in an infinitive, because the endings it restores are
conjugations. These suffixes attach to the *stem*, and the word list keeps a
great many verbs as bare stems (`verschieb/~~hY`, `absperr/~~*Vcej`), which that
filter skips entirely.

**The oracle.** The suffix is not productive for every verb — there is no
`*Wissung`, no `*Gehung` — so the decision is made per entry:

    hunspell must analyse the generated noun as a form of this very verb

    verschieb   -> Verschiebung   st:verschieben   -> add the flags
    wiss        -> Wissung        unknown          -> leave it alone

The stem hunspell reports is the infinitive, so it is the entry itself when the
entry is one, and the entry plus `-en`/`-n` when the entry is a bare stem.
Nothing looser: matching on a shared ending would accept `stell` for
`Bestellung`.

    scripts/add_german_verb_derivations.py              # report only
    scripts/add_german_verb_derivations.py --apply      # write dictionary.dict

Shares its oracle with `fix_german_verb_forms.py`; see
`german_dictionary_oracle.py` for the ISO-8859-1 trap it guards against.
"""

import collections
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

# The flags this script restores.
DERIVATION = {
    "7": "die Verschiebung",
    "8": "die Verschiebungen",
    "n": "das automatisierte",
    "x": "beeinflussbar",
}

# There is deliberately no flag filter. The word list keeps thousands of verbs
# under flags that say nothing about a verb -- `verschieb/~~hY` carries the noun
# plural and nothing else, and a filter on the verb flags skipped exactly the
# entries this script exists for. The oracle below is exact enough to stand on
# its own: the generated noun has to analyse to *this* entry's infinitive, which
# no unrelated headword satisfies by accident.


def capitalized(word):
    return word[:1].upper() + word[1:]


def is_form_of(entry, form, stems):
    """Does hunspell analyse `form` as a word built from this verb?

    The stem it reports is the infinitive: the entry itself when the entry is
    one, the entry plus `-en` or `-n` when the entry is a bare stem.
    """
    return any(
        stem.lower() in (entry, entry + "en", entry + "n")
        for stem in stems.get(form, ())
    )


def candidates(flag, rules, wanted):
    """Verb entries that should carry `flag` and do not, with the noun it adds."""
    out = []
    for index, word, flags, comment in entries():
        if not wanted(word):
            continue
        if flag in flagset(flags):
            continue
        generated = forms(rules, word)
        if generated:
            out.append((index, word, generated))
    return out


def main():
    parser = argument_parser(__doc__, "78nx")
    args = parser.parse_args()

    check_umlauts_survive(args.hunspell_dict)

    additions = collections.defaultdict(set)
    report = []

    for flag in args.flags:
        rules = affix_rules(flag)
        found = candidates(flag, rules, entry_filter(args.matching))
        # The '-ung' noun is written with a capital, the adjectives are not.
        cased = capitalized if flag in "78" else (lambda word: word)
        stems = analyse(
            (cased(form) for _, _, gen in found for form in gen),
            args.hunspell_dict,
        )
        accepted = [
            c
            for c in found
            if all(is_form_of(c[1], cased(form), stems) for form in c[2])
        ]
        for index, _, _ in accepted:
            additions[index].add(flag)

        report.append((flag, DERIVATION[flag], len(found), len(accepted)))
        if accepted:
            sample = ", ".join(f"{w} -> {g[0]}" for _, w, g in accepted[:4])
            print(f"  {flag} ({DERIVATION[flag]}): {len(accepted)}/{len(found)}  e.g. {sample}")

    print(f"\n{'suffix':26} {'missing':>9} {'confirmed':>10}")
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
