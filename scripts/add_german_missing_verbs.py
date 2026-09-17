#!/usr/bin/env python3
"""Add German verbs that are missing from the dictionary outright.

`dienen` and `zählen` are not in `dictionary.dict` at all — only the compounding
stems `diensts` and `zahl` are — so Harper reports "dienen", "diente" and
"zählte" as misspellings. They are not alone.

A word is added only if hunspell's expanded form list accepts the infinitive
*and* every finite form the conjugation rules would build from it. That is a
deliberately strict test: it is the same one
`scripts/add_german_verb_conjugation_flags.py` applies, so a verb that passes
here will pass there too.

It is not sufficient on its own, though. Strong preterite plurals are shaped
exactly like infinitives — `abbrachen`, `anlasen` — and their weak forms happen
to be accepted too. The headword list settles it: igerman98 lists infinitives and
derives the rest, so `zählen` and `dienen` are headwords in `de_DE.dic` and
`abbrachen` is not.

The entry is written with the bare verb property. Run the conjugation script
afterwards to fill in the affix flags:

    unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt
    scripts/add_german_missing_verbs.py --forms forms.txt --apply
    scripts/add_german_verb_conjugation_flags.py --forms forms.txt --apply

`--apply` writes; the default is a dry run. Re-running is idempotent.
"""

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from add_german_verb_conjugation_flags import (  # noqa: E402
    ANNOTATIONS,
    DICT,
    LIKELY_NOUN_PLURAL,
    forms_for,
    infinitive_shaped,
    load_rules,
)

# The verb property. Collision-free: `V` is not also an affix flag.
VERB_PROPERTY = "~~V"


def lemmas(dic: Path) -> set[str]:
    """Every headword in `de_DE.dic`.

    This is the test for "is it a lemma": igerman98 lists infinitives as
    headwords and derives the rest, so `zählen` and `dienen` are in here and
    `abbrachen`, `anlasen` and `amen` are not.

    It used to shell out to `hunspell -m` and read the `st:` field, which cannot
    work on this system: the shipped `de_DE.aff` declares `SET ISO8859-1` while
    the `.dic` beside it is UTF-8, so `hunspell` mis-decodes every umlaut. Sent
    as ISO-8859-1, `zählen` came back split into `z` and `hlen`; sent as UTF-8,
    as `zÃ¤hlen`. Either way no `st:` matched, and every umlaut verb was dropped
    in silence — which is why `zählen` was still missing after the first import.
    """
    raw = dic.read_bytes()
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError:
        text = raw.decode("iso-8859-1")

    return {
        line.split("/")[0].strip()
        for line in text.splitlines()
        if line.strip() and not line.startswith("#")
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="write the changes")
    parser.add_argument(
        "--forms",
        required=True,
        type=Path,
        help="hunspell form list from `unmunch` (see the module docstring)",
    )
    parser.add_argument(
        "--dic",
        type=Path,
        default=Path("/usr/share/hunspell/de_DE.dic"),
        help="hunspell source dictionary, read for its headwords",
    )
    args = parser.parse_args()

    for path in (DICT, ANNOTATIONS, args.forms, args.dic):
        if not path.exists():
            print(f"{path} not found -- run from the repo root", file=sys.stderr)
            return 1

    forms = {
        line.strip()
        for line in args.forms.read_text(encoding="utf-8", errors="replace").splitlines()
        if line.strip()
    }
    rules = load_rules(ANNOTATIONS)

    lines = DICT.read_text(encoding="utf-8").splitlines(keepends=True)
    known = {line.split("/", 1)[0].strip().lower() for line in lines if "/" in line}

    candidates = []
    for word in sorted(forms):
        if not word.islower() or not infinitive_shaped(word):
            continue
        if word in known or word in LIKELY_NOUN_PLURAL:
            continue
        tense_forms = {f for flag in "dei" for f in forms_for(word, rules[flag])}
        if not tense_forms or not all(f in forms for f in tense_forms):
            continue
        # A real verb form has no capitalized twin; see the note in the
        # conjugation script about lower-case noun forms in the list.
        if all(f.capitalize() in forms for f in tense_forms):
            continue
        candidates.append(word)

    before = len(candidates)
    headwords = lemmas(args.dic)
    candidates = [w for w in candidates if w in headwords]
    print(f"{before - len(candidates)} rejected: inflected forms, not headwords")

    print(f"{len(candidates)} verbs missing from the dictionary")
    for word in candidates[:15]:
        print(f"    {word}/{VERB_PROPERTY}")

    if not candidates:
        return 0

    if args.apply:
        with DICT.open("a", encoding="utf-8") as f:
            for word in candidates:
                f.write(f"{word}/{VERB_PROPERTY}\n")
        print(f"appended {len(candidates)} entries to {DICT}")
        print("now run scripts/add_german_verb_conjugation_flags.py --apply")
    else:
        print("dry run; pass --apply to write")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
