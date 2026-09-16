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
to be accepted too. Hunspell's own morphological analysis settles it: a lemma
analyses to itself (`dienen  st:dienen`), an inflected form to something else
(`abbrachen  st:abbrach fl:Z`). Candidates are put through `hunspell -m` and only
the lemmas survive.

The entry is written with the bare verb property. Run the conjugation script
afterwards to fill in the affix flags:

    unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt
    scripts/add_german_missing_verbs.py --forms forms.txt --apply
    scripts/add_german_verb_conjugation_flags.py --forms forms.txt --apply

`--apply` writes; the default is a dry run. Re-running is idempotent.
"""

import argparse
import subprocess
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

# The shipped de_DE dictionary declares ISO-8859-1 and hunspell believes it, so
# both directions have to be converted even though the file itself is UTF-8.
HUNSPELL_ENCODING = "iso-8859-1"


def lemmas(words: list[str]) -> set[str]:
    """Of `words`, the ones hunspell analyses as a lemma rather than a form.

    `dienen  st:dienen` is a lemma; `abbrachen  st:abbrach fl:Z` is the preterite
    plural of `abbrechen` wearing an infinitive's shape.
    """
    if not words:
        return set()

    payload = "\n".join(words).encode(HUNSPELL_ENCODING, errors="replace")
    try:
        result = subprocess.run(
            ["hunspell", "-d", "de_DE", "-m"],
            input=payload,
            capture_output=True,
            check=True,
        )
    except (OSError, subprocess.CalledProcessError) as err:
        raise SystemExit(f"could not run hunspell -m: {err}") from err

    out = result.stdout.decode(HUNSPELL_ENCODING, errors="replace")
    keep = set()
    for line in out.splitlines():
        line = line.strip()
        if not line or line.startswith("@(#)"):
            continue
        surface, _, analysis = line.partition("  ")
        for field in analysis.split():
            if field == f"st:{surface}":
                keep.add(surface)
    return keep


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

    for path in (DICT, ANNOTATIONS, args.forms):
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
    keep = lemmas(candidates)
    candidates = [w for w in candidates if w in keep]
    print(f"{before - len(candidates)} rejected by `hunspell -m`: inflected forms, not lemmas")

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
