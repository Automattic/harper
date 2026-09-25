#!/usr/bin/env python3
"""Check the gender on German noun entries against the corpus that uses them.

Gender in `dictionary.dict` is wrong often enough to be unusable. Measured over
the 4000 most frequent nouns of the prose corpus, roughly one entry in fourteen
that carries a gender carries the wrong one, and the mistakes are systematic:
`Leber`, `Mauer`, `Dauer`, `Nummer`, `Ziffer`, `Metapher` and `Kammer` are
feminine and recorded masculine, `Tier`, `Heer`, `Meer`, `Papier`, `Fenster`,
`Kloster`, `Gewitter` and `Wetter` are neuter and recorded masculine. An `-er`
that was read as an agent noun accounts for most of it.

That is why `german_preposition_case.rs` narrows a determiner by number only.
Gender cannot be read until it can be trusted.

**The oracle is the text, not another dictionary.** A German article names the
gender of the noun it introduces, and a handful of article forms do so without
any competing reading:

    eine, einer                     -> feminine
    einen                           -> masculine
    das                             -> neuter
    dem, des, einem, eines, diesem, dieses, keinem, keines,
    meinem, meines, seinem, seines, ihrem, ihres, jedem, jedes
                                    -> masculine or neuter, never feminine

Everything else is ambiguous and is left out. `dieser` is nominative masculine
*and* dative feminine; `keine` is feminine singular *and* plural; `keinen` is
accusative singular *and* dative plural. Including `dieser` alone was enough to
make *Zeit* come out masculine, on 129 votes.

Two things in the text look like the pattern and are not, and each was found by
reading the disagreements:

* **A hyphenated compound.** *das Kaiser-Wilhelm-Denkmal* votes for `Kaiser`
  unless the word after the noun is checked for a hyphen.
* **An indeclinable attributive adjective.** *das Londoner Abkommen* votes for
  `Londoner`, which is not the noun. A second capitalized word after the
  candidate rules it out.

Usage:

    audit_german_gender.py --corpus .archive/german-language/corpus-prose
    audit_german_gender.py --corpus <dir> --apply

Without `--apply` nothing is written. With it, only entries whose recorded
gender the corpus *contradicts* are rewritten; entries with no gender are left
alone, because adding one is a separate question with a separate error rate.
"""

import argparse
import collections
import pathlib
import re
import sys

FEMININE = {"eine", "einer"}
MASCULINE = {"einen"}
NEUTER = {"das"}
NOT_FEMININE = {
    "einem", "eines", "dem", "des", "diesem", "dieses", "keinem", "keines",
    "meinem", "meines", "seinem", "seines", "ihrem", "ihres", "jedem", "jedes",
}
CUES = FEMININE | MASCULINE | NEUTER | NOT_FEMININE

FLAG_OF = {"M": "M", "F": "F", "Z": "N"}
GENDER_FLAG = {"M": "M", "F": "F", "N": "Z"}

# cue, noun, and enough of what follows to see a hyphen or a second capital
PHRASE = re.compile(
    r"\b([A-Za-zÄÖÜäöüß]+)\s+([A-ZÄÖÜ][A-Za-zÄÖÜäöüß]*)(.?)\s*([A-ZÄÖÜ]?)"
)


def collect_votes(corpus: pathlib.Path) -> dict[str, collections.Counter]:
    votes: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
    for path in sorted(corpus.glob("*.md")):
        for cue, noun, after, then in PHRASE.findall(path.read_text(encoding="utf-8")):
            cue = cue.lower()
            if cue not in CUES:
                continue
            if after == "-" or then:
                continue
            if cue in FEMININE:
                votes[noun]["F"] += 1
            elif cue in MASCULINE:
                votes[noun]["M"] += 1
            elif cue in NEUTER:
                votes[noun]["N"] += 1
            else:
                votes[noun]["MN"] += 1
    return votes


def verdict(counts: collections.Counter, minimum: int) -> set[str] | None:
    """The gender the corpus agrees on, or `None` if it does not agree."""
    if sum(counts.values()) < minimum:
        return None
    feminine = counts.get("F", 0)
    masculine = counts.get("M", 0)
    neuter = counts.get("N", 0)
    either = counts.get("MN", 0)
    if feminine and not (masculine + neuter + either):
        return {"F"}
    if feminine:
        return None
    if neuter and not masculine:
        return {"N"}
    if masculine and not neuter:
        return {"M"}
    if either:
        return {"M", "N"}
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", required=True, type=pathlib.Path)
    parser.add_argument(
        "--dictionary",
        type=pathlib.Path,
        default=pathlib.Path(__file__).resolve().parents[1] / "dictionary.dict",
    )
    parser.add_argument("--min-votes", type=int, default=5)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()

    votes = collect_votes(args.corpus)
    lines = args.dictionary.read_text(encoding="utf-8").splitlines(keepends=True)

    agreed = 0
    conflicts: list[tuple[int, str, set[str], set[str], int]] = []
    for number, line in enumerate(lines):
        body = line.split("#", 1)[0].strip()
        if "/" not in body:
            continue
        word, flags = body.split("/", 1)
        recorded = {FLAG_OF[c] for c in flags if c in FLAG_OF}
        if not recorded:
            continue
        counts = votes.get(word[:1].upper() + word[1:])
        if counts is None:
            continue
        found = verdict(counts, args.min_votes)
        if found is None:
            continue
        if recorded & found:
            agreed += 1
        else:
            conflicts.append((number, word, recorded, found, sum(counts.values())))

    print(f"corpus agrees with {agreed} entries and contradicts {len(conflicts)}")
    for _, word, recorded, found, total in sorted(conflicts, key=lambda c: -c[4]):
        print(
            f"  {word:24} recorded {''.join(sorted(recorded)):3} "
            f"corpus {''.join(sorted(found)):3} ({total} votes)"
        )

    if not args.apply:
        print("\nnothing written; pass --apply to rewrite the contradicted entries")
        return 0

    for number, _, recorded, found, _ in conflicts:
        line = lines[number]
        body, hash_, comment = line.partition("#")
        word, flags = body.rstrip().split("/", 1)
        for gender in recorded:
            flags = flags.replace(GENDER_FLAG[gender], "", 1)
        flags += "".join(GENDER_FLAG[g] for g in sorted(found))
        rebuilt = f"{word}/{flags}"
        lines[number] = (
            f"{rebuilt} {hash_}{comment}" if hash_ else f"{rebuilt}\n"
        )

    args.dictionary.write_text("".join(lines), encoding="utf-8")
    print(f"\nrewrote {len(conflicts)} entries in {args.dictionary}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
