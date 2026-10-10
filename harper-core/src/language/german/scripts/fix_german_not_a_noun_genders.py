#!/usr/bin/env python3
"""Take the noun gender off the entries that are marked as not being nouns.

`strip_german_noun_readings.py` took the noun reading off 35623 lower-case
entries whose capitalized form igerman98 does not list, and wrote
`# not a noun: no capitalized form` behind each. It removed `N` and the plural
affixes, but not the gender properties `M`, `F` and `Z` — and those carry a noun
reading of their own (`"noun": {}` plus the gender). So `daher/~~Mh`,
`bisher/~~Mh`, `eher/~~Mh` and `hierzulande/~~FhA` stayed masculine and
feminine nouns, and `GermanNounCapitalization` asks for *Hierzulande* in *die
hierzulande übliche Technik*. The singular property `A` (also a noun reading)
was later added to some of them by the number pass, which read the gender.

This removes `M`, `F`, `Z`, `A` and `+` from exactly those entries and leaves
everything else on the line alone: the compound marker `h` and the verb
derivation flags (`7`, `8`, `n`, `x`) build forms that have nothing to do with
the noun reading.

    python3 harper-core/src/language/german/scripts/fix_german_not_a_noun_genders.py

Idempotent.
"""

import pathlib
import re

DICT = pathlib.Path(__file__).resolve().parent.parent / "dictionary.dict"
MARK = "# not a noun: no capitalized form"
NOUN_ONLY_PROPERTIES = set("MFZA+")
LINE = re.compile(r"^(?P<word>[^/\s]+)/(?P<tilde>~*)(?P<flags>\S*)(?P<rest>.*)$")


# The same entries also kept the noun-plural affix `Y` where it was the only
# source of a real word: `versprach/~~hY` builds `versprachen`. But `Y` gives
# its base a noun reading too, so *er versprach* wanted a capital. These are
# strong preterites and verb stems, so `Y` gives way to the verb property and
# the `-en` form it built becomes an entry of its own. Not for the three that
# are no verb.
NOT_VERBS = {"div", "hominid", "zehntausend"}


def main() -> None:
    lines = DICT.read_text(encoding="utf-8").splitlines(keepends=True)
    changed = 0
    verbs = []
    out = []
    for line in lines:
        match = LINE.match(line.rstrip("\n"))
        if match and MARK in match["rest"] and NOUN_ONLY_PROPERTIES & set(match["flags"]):
            flags = "".join(c for c in match["flags"] if c not in NOUN_ONLY_PROPERTIES)
            line = f"{match['word']}/{match['tilde']}{flags}{match['rest']}\n"
            match = LINE.match(line.rstrip("\n"))
            changed += 1
        if (
            match
            and MARK in match["rest"]
            and "Y" in match["flags"]
            and match["word"] not in NOT_VERBS
        ):
            flags = match["flags"].replace("Y", "") + "V"
            line = f"{match['word']}/{match['tilde']}{flags}{match['rest']}\n"
            verbs.append(f"{match['word']}en/~~V # verb form, was built by the noun affix Y\n")
        out.append(line)
    present = {line.partition("/")[0] for line in out}
    out += [line for line in verbs if line.partition("/")[0] not in present]
    DICT.write_text("".join(out), encoding="utf-8")
    print(f"{changed} entries lost a gender they had as non-nouns, {len(verbs)} became verbs")


if __name__ == "__main__":
    main()
