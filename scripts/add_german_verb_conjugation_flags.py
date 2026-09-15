#!/usr/bin/env python3
"""Give German infinitives their conjugation affix flags.

`dictionary.dict` stores the finite forms of a verb nowhere; they are generated
by the affix flags `d` (-te), `f` (-e), `i` (-t) and `j` (-en). An entry that
lacks them produces no finite forms at all, and Harper then reports "diente",
"promovierte" and "herrschte" as misspellings.

Which entries are verbs cannot be read off the dictionary itself: the `V`
property is missing from plenty of them ("promovieren/~~Nh", "herrschen/~~XZ")
and wrong on plenty of others, where a plural noun picked it up by accident.
So the decision is made against hunspell instead, which knows the inflected
forms this file is trying to produce. An entry is conjugated only if hunspell
accepts *every* form the four flags would generate.

The match is deliberately case sensitive, and that is what keeps noun plurals
out: German verbs are lower case, nouns are capitalized. "bären" would otherwise
be conjugated to "bärte" on the strength of hunspell's "Bärte" -- the plural of
"Bart", not a verb form at all.

Generate the form list once (it is large, and not committed):

    unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt

Run from the repo root:

    scripts/add_german_verb_conjugation_flags.py --forms forms.txt [--apply]

`--apply` writes; the default is a dry run. Re-running is idempotent.
"""

import argparse
import sys
from pathlib import Path

DICT = Path("harper-core/src/language/german/dictionary.dict")

# The conventional conjugation set: -te, -e, -t, -en. Matches the 22592 entries
# that already have it. `h` (-st) is deliberately not added: it doubles as the
# "vetted" bookkeeping marker on noun entries, so it is not a clean verb signal.
CONJUGATION = "dfij"

# Any of these means the entry already generates finite forms.
AFFIX_FLAGS = set("cdefij")

# Noun plurals mistagged `V`. Hand-audited: each is the plural of a common noun
# and has no verb reading at all.
LIKELY_NOUN_PLURAL = {
    "bären",
    "besen",
    "bissen",
    "boten",
    "damen",
    "daten",
    "farben",
    "fasern",
    "ferien",
    "flocken",
    "gaben",
    "garten",
    "gassen",
    "gnaden",
    "graben",
    "haufen",
    "hosen",
    "kosten",
    "kunden",
    "laden",
    "lagen",
    "listen",
    "massen",
    "noten",
    "quellen",
    "raten",
    "reihen",
    "rosen",
    "sachen",
    "samen",
    "seiten",
    "sorten",
    "stufen",
    "tanten",
    "themen",
    "tonnen",
    "waren",
    "wellen",
    "wochen",
    "zahlen",
    "zeiten",
}


def stem_of(word: str) -> str:
    """The conjugation stem the affix rules expect to build on."""
    if word.endswith(("ern", "eln")):
        return word[:-1]
    if word.endswith("en"):
        return word[:-2]
    return word


def generated_forms(word: str) -> list[str]:
    """The surface forms flags `d`, `f`, `i` and `j` would add for `word`.

    `j` is the odd one out: for an `-ern`/`-eln` verb the infinitive already *is*
    the plural, so it restores the `n` instead of appending `en`.
    """
    stem = stem_of(word)
    plural = stem + "n" if word.endswith(("ern", "eln")) else stem + "en"
    return [stem + "te", stem + "e", stem + "t", plural]


def should_conjugate(word: str, flags: str, forms: set[str]) -> bool:
    if AFFIX_FLAGS & set(flags):
        return False
    if not word.endswith(("en", "ern", "eln")):
        return False
    if word.lower() in LIKELY_NOUN_PLURAL:
        return False
    # Three is enough for "hören", "reden", "baden". Anything shorter leaves too
    # little for the oracle to discriminate on.
    if len(stem_of(word)) < 3:
        return False
    # Case sensitive on purpose: a capitalized hunspell form is a noun, and
    # accepting it here is exactly how noun plurals sneak in.
    return all(form in forms for form in generated_forms(word))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="write the changes")
    parser.add_argument(
        "--forms",
        required=True,
        type=Path,
        help="hunspell form list from `unmunch` (see the module docstring)",
    )
    args = parser.parse_args()

    if not DICT.exists():
        print(f"{DICT} not found -- run from the repo root", file=sys.stderr)
        return 1

    if not args.forms.exists():
        print(f"{args.forms} not found", file=sys.stderr)
        return 1

    forms = {
        line.strip()
        for line in args.forms.read_text(encoding="utf-8", errors="replace").splitlines()
        if line.strip()
    }
    print(f"{len(forms)} hunspell forms loaded from {args.forms}")

    out = []
    changed = 0
    samples = []

    for line in DICT.read_text(encoding="utf-8").splitlines(keepends=True):
        body, sep, comment = line.partition("#")
        stripped = body.strip()
        if not stripped or "/" not in stripped:
            out.append(line)
            continue

        word, _, flags = stripped.partition("/")
        if not should_conjugate(word, flags.replace("~", ""), forms):
            out.append(line)
            continue

        missing = "".join(c for c in CONJUGATION if c not in flags)
        if not missing:
            out.append(line)
            continue

        new_body = f"{word}/{flags}{missing}"
        # Keep the original spacing between the entry and any trailing comment.
        trailing = body[len(body.rstrip()) :]
        out.append(f"{new_body}{trailing}{sep}{comment}")
        changed += 1
        if len(samples) < 15:
            samples.append(f"{stripped}  ->  {new_body}")

    print(f"{changed} entries gain '{CONJUGATION}'")
    for s in samples:
        print(f"    {s}")

    if args.apply:
        DICT.write_text("".join(out), encoding="utf-8")
        print(f"wrote {DICT}")
    else:
        print("dry run; pass --apply to write")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
