#!/usr/bin/env python3
"""Record every gender of the German nouns that have more than one.

`GermanDeterminerGender` reports an article that fits none of the genders an
entry records, so an entry that records one gender of two turns the other,
correct, phrase into a report: *das Teil* against an entry that says *der Teil*.
The data-side half of that rule is this list, taken from the nouns Duden marks
with several genders (*der/das Teil*, *der/die See*, *der/das Schild*).

Gender flags are plain properties (`M`, `F`, `Z`) and not affixes, so adding one
cannot build a surface form. The script is idempotent.

    python3 harper-core/src/language/german/scripts/add_german_multi_gender_nouns.py
"""

import pathlib
import re
import sys

DICT = pathlib.Path(__file__).resolve().parent.parent / "dictionary.dict"

# lemma as it appears at the start of the entry line -> genders to guarantee.
# M masculine, F feminine, Z neuter.
GENDERS = {
    "Teil": "MZ",  # der Teil / das Teil
    "see": "MF",  # der See / die See
    "band": "MFZ",  # der Band / die Band / das Band
    "leiter": "MF",  # der Leiter / die Leiter
    "kiefer": "MF",  # der Kiefer / die Kiefer
    "tor": "MZ",  # der Tor / das Tor
    "erbe": "MZ",  # der Erbe / das Erbe
    "verdienst": "MZ",  # der Verdienst / das Verdienst
    "schild": "MZ",  # der Schild / das Schild
    "moment": "MZ",  # der Moment / das Moment
    "joghurt": "MFZ",  # der / die / das Joghurt
    "mangel": "MF",  # der Mangel / die Mangel
    "hut": "MF",  # der Hut / die Hut
    "kunde": "MF",  # der Kunde / die Kunde
    "heide": "MF",  # der Heide / die Heide
    "stift": "MZ",  # der Stift / das Stift
    "mark": "FZ",  # die Mark / das Mark
    "reis": "MZ",  # der Reis / das Reis
    "tau": "MZ",  # der Tau / das Tau
    "virus": "MZ",  # das Virus / der Virus
    "meter": "MZ",  # der Meter / das Meter
    "liter": "MZ",  # der Liter / das Liter
    "keks": "MZ",  # der Keks / das Keks
}

LINE = re.compile(r"^(?P<word>[^/\s]+)/(?P<head>~*)(?P<flags>\S*)(?P<rest>.*)$")


def main() -> int:
    lines = DICT.read_text(encoding="utf-8").split("\n")
    changed = 0
    for i, line in enumerate(lines):
        m = LINE.match(line)
        if not m or m["word"] not in GENDERS:
            continue
        flags = m["flags"]
        # Only the noun entry: a verb or adjective homograph keeps its flags.
        if "N" not in flags and not set("MFZ") & set(flags):
            continue
        missing = "".join(g for g in GENDERS[m["word"]] if g not in flags)
        if missing:
            lines[i] = f"{m['word']}/{m['head']}{flags}{missing}{m['rest']}"
            changed += 1
    DICT.write_text("\n".join(lines), encoding="utf-8")
    print(f"updated {changed} entries", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
