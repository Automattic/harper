#!/usr/bin/env python3
"""Give the `A` flag to noun entries that have no number at all.

A German noun gets its number from the plural affix it carries: `Frau` plus `Y`
builds `Frauen`, and the affix marks the base singular in return. A noun whose
plural this dictionary cannot build carries no plural affix and so has no
number — `bruder`, `vater`, and the rest of the umlaut plurals, 30000 entries of
them.

That is not cosmetic. `den` is accusative masculine singular *or* dative plural,
and only the noun can say which. A noun that says nothing leaves the dative
reading standing, and *mit den Bruder* passes.

**The oracle is igerman98**, through `hunspell -m`: a word it analyses as
`st:<itself>` with no plural flag is a base form, and a base form of a noun is a
singular. Words it does not know at all are left alone. Note that `de_DE.aff`
declares `SET ISO8859-1` while the `.dic` beside it is UTF-8, so every word with
an umlaut fails to analyse *quietly* — copy the `.aff`, rewrite that one line to
`SET UTF-8`, and leave the `.dic` alone.

Two classes are held back even when the oracle calls them base forms:

* **`-er`, `-el` and `-en` nouns**, whose plural is spelled like the singular —
  *der Lehrer*, *die Lehrer*. Neither number alone is right, and claiming one
  would invent errors on correct text.
* **Pluralia tantum** — *Eltern*, *Masern*, *Jeans*. A closed class, listed
  below, because nothing in the morphology distinguishes them.
* **Latin and Greek plurals** — *Korpora*, *Charakteristika*, *Pronomina*,
  *Termini*, *Visa*. hunspell reads every one as a base form, because that is
  what they are in its word list.

Usage:

    mark_german_singular_nouns.py --hunspell <dir>/de_utf [--apply]
"""

import argparse
import collections
import pathlib
import re
import subprocess
import sys
import tempfile

# igerman98 suffix flags that build a plural.
PLURAL_FLAGS = set("ENPRpq")

# Spelled the same in the singular and the plural.
SAME_IN_BOTH_NUMBERS = ("er", "el", "en")

# A Latin or Greek plural: Korpora, Charakteristika, Pronomina, Neutra, Visa,
# Termini. hunspell reads every one of them as a base form, because that is what
# they are in its word list, and marking them singular reported *zu den
# Korpora* and six more like it. The cost is the singulars that end the same
# way -- Kamera, Pizza, Oma -- which keep no number at all.
FOREIGN_PLURAL = ("a", "i")

# No singular at all. Nothing in the morphology says so, so they are named.
PLURALIA_TANTUM = {
    "eltern", "geschwister", "leute", "ferien", "masern", "pocken", "röteln",
    "kosten", "möbel", "lebensmittel", "trümmer", "jeans", "shorts", "alpen",
    "anden", "kanaren", "niederlande", "philippinen", "usa", "spesen",
    "einkünfte", "finanzen", "gliedmaßen", "personalien", "wirren", "zinsen",
}

WORD = re.compile(r"^[A-Za-zÄÖÜäöüß]+$")


def analyse(words: list[str], dictionary: pathlib.Path) -> dict[str, list[tuple]]:
    """`hunspell -m` for every word, as {word: [(stem, flag), ...]}."""
    with tempfile.NamedTemporaryFile("w", suffix=".txt", encoding="utf-8") as handle:
        handle.write("\n".join(words) + "\n")
        handle.flush()
        output = subprocess.run(
            ["hunspell", "-d", str(dictionary), "-m", handle.name],
            capture_output=True, text=True, check=True,
        ).stdout

    readings: dict[str, list[tuple]] = collections.defaultdict(list)
    for line in output.splitlines():
        if not line.strip():
            continue
        word = line.split(None, 1)[0]
        stem = re.search(r"st:(\S+)", line)
        flag = re.search(r"fl:(\S+)", line)
        readings[word].append(
            (stem.group(1) if stem else None, flag.group(1) if flag else None)
        )
    return readings


def is_a_singular(word: str, readings: list[tuple]) -> bool:
    if not any(stem for stem, _ in readings):
        return False
    for stem, flag in readings:
        if flag and stem and stem.lower() != word.lower():
            if any(c in PLURAL_FLAGS for c in flag):
                return False
    return any(flag is None or (stem and stem.lower() == word.lower())
               for stem, flag in readings)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--hunspell", required=True, type=pathlib.Path,
                        help="dictionary base name, without .aff/.dic")
    parser.add_argument(
        "--dictionary", type=pathlib.Path,
        default=pathlib.Path(__file__).resolve().parents[1] / "dictionary.dict",
    )
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()

    lines = args.dictionary.read_text(encoding="utf-8").splitlines(keepends=True)

    candidates: list[tuple[int, str]] = []
    for number, line in enumerate(lines):
        body = line.split("#", 1)[0].strip()
        if "/" not in body:
            continue
        word, flags = body.split("/", 1)
        if not WORD.match(word):
            continue
        if not set(flags) & set("NMFZ"):
            continue
        if set(flags) & set("XYabEA"):
            continue
        if word.lower().endswith(SAME_IN_BOTH_NUMBERS):
            continue
        if word.lower().endswith(FOREIGN_PLURAL):
            continue
        if word.lower() in PLURALIA_TANTUM:
            continue
        candidates.append((number, word))

    heads = sorted({word[:1].upper() + word[1:] for _, word in candidates})
    print(f"{len(candidates)} entries have a noun reading and no number")
    readings = analyse(heads, args.hunspell)

    singular = [
        (number, word) for number, word in candidates
        if is_a_singular(word[:1].upper() + word[1:],
                         readings.get(word[:1].upper() + word[1:], []))
    ]
    print(f"{len(singular)} of them igerman98 confirms as a base form")
    print("  e.g. " + ", ".join(word for _, word in singular[:12]))

    if not args.apply:
        print("\nnothing written; pass --apply")
        return 0

    for number, _ in singular:
        body, hash_, comment = lines[number].partition("#")
        rebuilt = body.rstrip() + "A"
        lines[number] = f"{rebuilt} {hash_}{comment}" if hash_ else f"{rebuilt}\n"

    args.dictionary.write_text("".join(lines), encoding="utf-8")
    print(f"\nmarked {len(singular)} entries singular in {args.dictionary}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
