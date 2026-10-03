#!/usr/bin/env python3
"""Give a noun reading to the German forms that are nouns too.

Harper reads `knoten`, `bogen`, `tat` and `last` as verb forms only, and has no
reading at all for the umlaut plural `räume`. Each is also a noun — *der
Knoten*, *der Bogen*, *die Tat*, *die Last*, *die Räume* — and the missing
reading matters wherever a rule asks whether a word can be a noun.
`can_head_a_capitalized_compound` asks exactly that of the last element of a
capitalized compound, so without this pass `Zeiträume`, `Lymphknoten`,
`Straftat` and `Steuerlast` are rejected along with `Bettrieb`.

The oracle is `hunspell -m` on the **capitalized** spelling, and the decision is
made by the stem it reports:

    Knoten   st:knoten  st:Knoten     the noun Knoten is a headword  -> noun
    Räume    st:räumen  st:Raum       a form of the headword Raum    -> noun
    Abstillen st:abstillen st:Abstillen  no headword `Abstillen`     -> not a noun
    Rieb     st:rieb                  no capitalized stem at all     -> not a noun

A capitalized stem alone is not enough, because hunspell nominalizes every
infinitive (`st:Abstillen`); the stem must also be a capitalized headword of
the hunspell dictionary. frami lists many nominalized infinitives as headwords
too (`Abarbeiten/SJm`), so when the lower-case form is an infinitive the
headword must also carry a compound flag (`i` or `j`), which the real noun
homographs do and the nominalizations do not:

    Knoten/Smij     Bogen/Smij      kept: nouns that are also verbs
    Abarbeiten/SJm  Essen/Sm        skipped: das Abarbeiten, das Essen Only forms Harper reads with neither a noun nor an
adjective reading are examined, and the reading is added as an entry of its own
(`knoten/~~N`) rather than as a flag on the verb, so the verb's affixes do not
turn `knotete` into a noun as well.

    harper-core/src/language/german/scripts/add_german_noun_homograph_readings.py --hunspell-dict <UTF-8 de_DE>
    ... --apply      # append the entries to dictionary.dict

Needs a release `harper-cli` built with `--features de`.
"""

import pathlib
import subprocess

from german_dictionary_oracle import (
    DICT,
    ROOT,
    analyse,
    argument_parser,
    check_umlauts_survive,
)

HARPER_CLI = ROOT / "target/release/harper-cli"


def harper_forms():
    """Every lower-case form in Harper's German word list."""
    out = subprocess.run(
        [HARPER_CLI, "words", "-d", "de"], capture_output=True, check=True, text=True
    ).stdout
    words = (line.strip().strip('"') for line in out.splitlines())
    return sorted(w for w in words if w.isalpha() and w == w.lower())


def without_nominal_reading(forms):
    """The forms Harper reads with neither a noun nor an adjective reading."""
    out = subprocess.run(
        [HARPER_CLI, "metadata", "-b", "-d", "de"],
        input="\n".join(forms) + "\n",
        capture_output=True,
        check=True,
        text=True,
    ).stdout
    found = []
    for line in out.splitlines():
        word, _, tags = line.partition(": ")
        letters = tags.split(" ")[0]
        if "N" not in letters and "J" not in letters:
            found.append(word)
    return found


def hunspell_headwords(dictionary):
    """The capitalized headwords of the hunspell dictionary, with their flags."""
    path = pathlib.Path(f"{dictionary}.dic")
    heads = {}
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines()[1:]:
        word, _, flags = line.strip().partition("/")
        if word[:1].isupper():
            heads[word] = heads.get(word, "") + flags
    return heads


def is_noun(form, headword, flags, infinitives):
    """Is `headword` a noun `form` belongs to, rather than its nominalization?"""
    if form not in infinitives:
        return True
    return "i" in flags or "j" in flags


def main():
    parser = argument_parser(__doc__, "")
    args = parser.parse_args()
    check_umlauts_survive(args.hunspell_dict)

    candidates = without_nominal_reading(harper_forms())
    stems = analyse((c[:1].upper() + c[1:] for c in candidates), args.hunspell_dict)
    heads = hunspell_headwords(args.hunspell_dict)
    lower = analyse(candidates, args.hunspell_dict)
    # `verbergen` analyses as `st:bergen`: the stem may be the unprefixed core.
    infinitives = {
        c
        for c in candidates
        if any(s.islower() and c.endswith(s) for s in lower.get(c, ()))
    }

    additions = []
    for form in candidates:
        nouns = sorted(
            s
            for s in stems.get(form[:1].upper() + form[1:], ())
            if s in heads and is_noun(form, s, heads[s], infinitives)
        )
        if nouns:
            additions.append((form, nouns[0]))

    print(f"{len(candidates)} forms have no noun or adjective reading")
    print(f"{len(additions)} of them are forms of a hunspell noun")
    for form, noun in additions[:40]:
        print(f"  {form:24} {noun}")

    if not args.apply:
        print("dry run; pass --apply to write")
        return

    with DICT.open("a", encoding="utf-8") as out:
        for form, noun in additions:
            out.write(f"{form}/~~N # noun reading: a form of {noun} (hunspell)\n")
    print(f"appended {len(additions)} entries to {DICT}")


if __name__ == "__main__":
    main()
