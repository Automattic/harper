#!/usr/bin/env python3
"""Restore the plurals and genitives German nouns are missing from the word list.

A German noun entry carries one flag per ending it may take: `X` for the plural
in *-e* (*Experiment* -> *Experimente*), `Y` for *-en*, `E` for *-n*, `a` for
*-er*/*-ern*, and `H` or `0` for the genitive in *-s* / *-es* (*des
Christentums*). Most entries carry none of them:

    87% of the noun entries have no genitive
    53% have no plural

The forms do not then exist for Harper. They are not reported as misspellings
only because the compound decomposition is permissive enough to cut
`Experimente` into pieces that happen to be words — which is the same
permissiveness that lets `vieleicht` through as `viel` + `eicht`. Every attempt
to tighten the decomposition runs straight into these holes: it starts
reporting `Experimente`, `Christentums` and `Amphitheaters`, and the tightening
looks like a regression when it is really the missing flags surfacing.

**The risk is adding an ending to something that is not a noun.** The word
list tags adjectives and verb stems `#noun` all over (`abundant/~~NXh #noun`,
`absperr/~~NXh`), so the flags only narrow the field and the decision is made
by an oracle. German writes its nouns with a capital, and so does hunspell's
dictionary, which is what makes the question askable:

    the entry, capitalized, must analyse to a capitalized stem
    and every form the ending would generate must analyse to that same stem

    Experiment    -> st:Experiment    a noun
    Experimente   -> st:Experiment    its plural          -> add the ending
    Abundant      -> st:abundant      an adjective        -> leave it alone
    Absperre      -> st:absperren     a form of the verb  -> leave it alone

The lower-case stem is how hunspell reports a word that is only capitalized
because a sentence began with it.

    scripts/fix_german_noun_forms.py              # report only
    scripts/fix_german_noun_forms.py --apply      # write dictionary.dict

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
    flagset,
    forms,
)


def capitalized(word):
    return word[:1].upper() + word[1:]


def is_form_of(entry, form, stems):
    """Does `form` analyse to the noun `entry`?

    Case carries the part of speech here: a capitalized stem is a noun,
    a lower-case one an adjective or a verb that happened to be capitalized.
    The stem may also be the entry's unprefixed core, the way hunspell
    analyses `Bundesministeriums` under `Ministerium`.
    """
    return any(
        stem[:1].isupper() and (stem == entry or entry.endswith(stem))
        for stem in stems.get(form, ())
    )

# The endings this script can restore, in the order they are reported.
DECLENSION = {
    "X": "die Experimente",
    "Y": "die Studenten",
    "E": "die Fragen",
    "a": "die Bilder",
    "H": "des Christentums",
    "0": "des Hauses",
}

# Flags that say "this entry is a noun".
NOUN_FLAGS = set("NMFZz")

# Flags that say the entry is an inflected form rather than a base noun. A
# plural does not take a second plural, and a declined adjective is not a noun
# at all. The verb flags are deliberately absent: German nominalizes every
# infinitive, and plenty of real nouns carry a verb reading as well — the
# oracle is what separates those.
INFLECTED_FLAGS = set("OQRSTUW")


def candidates(flag, rules):
    """Noun entries that should carry `flag` and do not, with the forms it adds."""
    out = []
    for index, word, flags, comment in entries():
        flags_present = flagset(flags)
        if flag in flags_present:
            continue
        if not flags_present & NOUN_FLAGS or flags_present & INFLECTED_FLAGS:
            continue
        generated = forms(rules, word)
        if generated:
            out.append((index, word, flags, comment, generated))
    return out


def noun_entries(found, dictionary):
    """Of `found`, the entries hunspell recognises as nouns in their own right."""
    lemmas = {capitalized(word) for _, word, _, _, _ in found}
    stems = analyse(lemmas, dictionary)
    return {lemma for lemma in lemmas if is_form_of(lemma, lemma, stems)}


def main():
    # `a` (die Bilder) is left out of the default: the 77 entries it confirms
    # are nominalized adjectives (`die Apuanische` -> `Apuanischer`), where the
    # string is German but calling it a noun plural is not.
    parser = argument_parser(__doc__, "XYEH0")
    args = parser.parse_args()

    check_umlauts_survive(args.hunspell_dict)

    additions = collections.defaultdict(set)
    report = []

    for flag in args.flags:
        rules = affix_rules(flag)
        found = candidates(flag, rules)
        nouns = noun_entries(found, args.hunspell_dict)
        found = [c for c in found if capitalized(c[1]) in nouns]
        stems = analyse(
            (capitalized(f) for _, _, _, _, gen in found for f in gen),
            args.hunspell_dict,
        )

        # Every form the ending builds must be a form of *this* entry. The `a`
        # class builds two (`Bilder`, `Bildern`); getting one of them wrong
        # would add the wrong one along with the right one.
        accepted = [
            c
            for c in found
            if all(is_form_of(capitalized(c[1]), capitalized(f), stems) for f in c[4])
        ]
        for index, _, _, _, _ in accepted:
            additions[index].add(flag)

        report.append((flag, DECLENSION[flag], len(found), len(accepted)))
        if accepted:
            sample = ", ".join(f"{w} -> {g[0]}" for _, w, _, _, g in accepted[:4])
            print(f"  {flag} ({DECLENSION[flag]}): {len(accepted)}/{len(found)}  e.g. {sample}")

    print(f"\n{'ending':26} {'missing':>9} {'confirmed':>10}")
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
