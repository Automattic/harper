#!/usr/bin/env python3
"""Import the adjectives hunspell declines that are not entries at all.

`fix_german_adjective_forms.py` repairs entries that exist but lack their
declension. This is the other half: adjectives with no entry, which Harper only
knows in their bare form because the compound decomposition cuts them into
words it has. `mangelhaft` passes as `Mangel` + `haft`, but nothing builds
`mangelhafter`, so a pupil's

    Die Arbeit war mangelhafter als die letzte.

is reported as a misspelling, and so are `ungenügenden`, `stümperhafte` and
four thousand others. A school grade is the last word that should be flagged.

**Hunspell decides what is an adjective** -- its `SFX A` declension class -- and
every form an ending would build must analyse back to that very headword, the
check `fix_german_adjective_forms.py` uses. An entry is written with the
endings that pass:

    mangelhaft/~~hJOQRST

and nothing else: comparison (`U`, `W`) and the `un-` prefix are left to the
existing repair scripts, which verify them the same way.

**Short headwords are skipped** (`--min-length`, default 6). Every entry of three
characters or more becomes a compound element, and a short adjective is also a
plausible typo of a frequent word -- see the dictionary-growth notes in the
language README.

    harper-core/src/language/german/scripts/add_german_missing_adjectives.py --hunspell-dict PATH            # report
    harper-core/src/language/german/scripts/add_german_missing_adjectives.py --hunspell-dict PATH --apply    # write

PATH is a UTF-8 copy of de_DE (without extension); see
`german_dictionary_oracle.py` for why the system one does not work. Re-running
is idempotent.
"""

import sys

from fix_german_adjective_forms import adjective_headwords, is_form_of
from german_dictionary_oracle import (
    DICT,
    affix_rules,
    analyse,
    argument_parser,
    check_umlauts_survive,
    entries,
    forms,
)

ADJECTIVE_PROPERTY = "J"
COMPOUND_MARKER = "h"


def main():
    parser = argument_parser(__doc__, "OQRST")
    parser.add_argument("--min-length", type=int, default=6)
    args = parser.parse_args()

    check_umlauts_survive(args.hunspell_dict)
    known = {word.lower() for _, word, _, _ in entries()}
    candidates = sorted(
        word
        for word in adjective_headwords(args.hunspell_dict)
        if word not in known and word.isalpha() and len(word) >= args.min_length
    )
    print(f"{len(candidates)} adjectives hunspell declines have no entry")

    rules = {flag: affix_rules(flag) for flag in args.flags}
    generated = {
        word: {flag: forms(rules[flag], word) for flag in args.flags}
        for word in candidates
    }
    stems = analyse(
        (f for by_flag in generated.values() for fs in by_flag.values() for f in fs),
        args.hunspell_dict,
    )

    accepted = []
    for word in candidates:
        flags = "".join(
            flag
            for flag, fs in generated[word].items()
            if fs and all(is_form_of(word, f, stems) for f in fs)
        )
        # An adjective none of whose declined forms hunspell confirms is an
        # indeclinable (`lila`) or a stem it lists for compounding only.
        if flags:
            accepted.append((word, flags))

    print(f"{len(accepted)} confirmed, e.g.")
    for word, flags in accepted[:: max(1, len(accepted) // 15)][:15]:
        print(f"    {word}/~~{COMPOUND_MARKER}{ADJECTIVE_PROPERTY}{flags}")

    if not args.apply:
        print("dry run; pass --apply to write")
        return 0

    with DICT.open("a", encoding="utf-8") as handle:
        for word, flags in accepted:
            handle.write(
                f"{word}/~~{COMPOUND_MARKER}{ADJECTIVE_PROPERTY}{flags} # igerman98 SFX A headword\n"
            )
    print(f"appended {len(accepted)} entries to {DICT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
