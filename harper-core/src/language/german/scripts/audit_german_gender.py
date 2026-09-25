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

**A second source, for the words the corpus cannot reach.** A few derivational
suffixes settle the gender on their own, and they were measured against the
corpus rather than assumed:

    -ung 99.5%   -tion -sion -ie -ismus -nis -ment 100%   -chen 92%   -um 90%

    -e 65%   -er 61%   -el 45%

Only the first row is used. The second is the rule that produced the damage in
the first place: an `-er` read as an agent-noun suffix is what made `Leber`,
`Mauer`, `Ziffer` and `Metapher` masculine.

**A third opinion, for validation only.** In igerman98 a headword with the
genitive `-es` flag `T` is never feminine, and one carrying `F` or `g` — the
`-in` derivation — is masculine, since a feminine does not derive a feminine.
That is sharp enough to contradict, not sharp enough to decide: it mislabels
`-ismus` nouns, whose genitive is uninflected, and umlaut-plural compound
elements. Disagreements are printed; nothing is written from it.

Note the trap in the dictionary itself: `de_DE.aff` declares `SET ISO8859-1`
while the `.dic` beside it is UTF-8, so every word with an umlaut fails to
analyse *quietly*. Copy the `.aff`, rewrite that one line, leave the `.dic`.

Usage:

    audit_german_gender.py --corpus .archive/german-language/corpus-prose
    audit_german_gender.py --corpus <dir> --apply   # fix contradicted genders
    audit_german_gender.py --corpus <dir> --add     # fill in missing ones

Without `--apply` or `--add` nothing is written.
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

# Only the suffixes that scored at least 90% against the corpus. `-e`, `-er` and
# `-el` are left out on purpose; see the module docstring.
SUFFIX_GENDER = [
    ("ung", "F"), ("heit", "F"), ("keit", "F"), ("schaft", "F"), ("ität", "F"),
    ("tion", "F"), ("sion", "F"), ("ie", "F"),
    ("ismus", "M"),
    ("chen", "N"), ("lein", "N"), ("ment", "N"), ("nis", "N"), ("tum", "N"),
]

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
    """The gender the corpus agrees on, or `None` if it does not agree.

    A *specific* gender needs that many votes of its own. Most of the evidence
    is `dem`/`des`/`einem`, which says only "not feminine", and letting one
    stray `das` outvote three of those made *Nutzer* neuter. Where the specific
    evidence is thin the answer is the weaker one, which is still worth having:
    ruling out the feminine rules out a third of the possibilities.
    """
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
    if neuter >= minimum and not masculine:
        return {"N"}
    if masculine >= minimum and not neuter:
        return {"M"}
    if masculine and neuter:
        return None
    return {"M", "N"}


def suffix_gender(word: str) -> str | None:
    """The gender a reliable derivational suffix settles, if any."""
    lower = word.lower()
    for suffix, gender in sorted(SUFFIX_GENDER, key=lambda pair: -len(pair[0])):
        if lower.endswith(suffix) and len(lower) > len(suffix) + 2:
            return gender
    return None


def igerman98_flags(path: pathlib.Path) -> dict[str, set[str]]:
    """Headword -> affix flags, from a hunspell `.dic`."""
    flags: dict[str, set[str]] = {}
    with path.open(encoding="utf-8") as handle:
        next(handle, None)
        for line in handle:
            word, _, rest = line.strip().partition("/")
            if word and word[:1].isalpha():
                flags.setdefault(word, set()).update(rest)
    return flags


def igerman98_objects(flags: set[str], found: set[str]) -> str | None:
    """Why igerman98 disagrees with `found`, or `None` if it does not."""
    if "T" in flags and found == {"F"}:
        return "genitive -es, so not feminine"
    if flags & {"F", "g"} and found == {"F"}:
        return "derives an -in form, so masculine"
    return None


def report_independent_check(votes, minimum: int) -> None:
    """Corpus against the suffixes, which are an independent source."""
    checked = correct = 0
    for word, counts in votes.items():
        expected = suffix_gender(word)
        found = verdict(counts, minimum)
        if expected is None or found is None:
            continue
        checked += 1
        correct += expected in found
    if checked:
        print(
            f"corpus against the derivational suffixes: {correct}/{checked} "
            f"({correct / checked:.1%})"
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", required=True, type=pathlib.Path)
    parser.add_argument(
        "--dictionary",
        type=pathlib.Path,
        default=pathlib.Path(__file__).resolve().parents[1] / "dictionary.dict",
    )
    parser.add_argument(
        "--igerman98",
        type=pathlib.Path,
        help="a hunspell .dic to cross-check against; disagreements are "
             "reported and nothing is written from it",
    )
    parser.add_argument("--min-votes", type=int, default=3)
    parser.add_argument(
        "--apply", action="store_true", help="rewrite contradicted genders"
    )
    parser.add_argument(
        "--add", action="store_true", help="fill in genders that are missing"
    )
    args = parser.parse_args()

    votes = collect_votes(args.corpus)
    print(f"{len(votes)} words carry at least one article vote")
    report_independent_check(votes, args.min_votes)

    foreign = igerman98_flags(args.igerman98) if args.igerman98 else {}
    lines = args.dictionary.read_text(encoding="utf-8").splitlines(keepends=True)

    agreed = 0
    conflicts: list[tuple[int, str, set[str], set[str], int]] = []
    additions: list[tuple[int, str, set[str], str]] = []
    objections: list[tuple[str, set[str], str]] = []
    added_words: set[str] = set()

    for number, line in enumerate(lines):
        body = line.split("#", 1)[0].strip()
        if "/" not in body:
            continue
        word, flags = body.split("/", 1)
        recorded = {FLAG_OF[c] for c in flags if c in FLAG_OF}
        headword = word[:1].upper() + word[1:]
        counts = votes.get(headword)
        found = verdict(counts, args.min_votes) if counts else None

        if recorded:
            if found is None:
                continue
            if recorded & found:
                agreed += 1
            else:
                conflicts.append((number, word, recorded, found, sum(counts.values())))
            continue

        # No gender yet. Only a plain noun entry gets one; a verb or an
        # adjective that merely shares the spelling must not.
        if "N" not in flags:
            continue
        source = "corpus"
        if found is None:
            suffix = suffix_gender(word)
            if suffix is None:
                continue
            found, source = {suffix}, "suffix"
        if objection := igerman98_objects(foreign.get(headword, set()), found):
            objections.append((word, found, objection))
            continue
        additions.append((number, word, found, source))
        added_words.add(word)

    print(f"\ncorpus agrees with {agreed} entries and contradicts {len(conflicts)}")
    for _, word, recorded, found, total in sorted(conflicts, key=lambda c: -c[4]):
        print(
            f"  {word:24} recorded {''.join(sorted(recorded)):3} "
            f"corpus {''.join(sorted(found)):3} ({total} votes)"
        )

    from_corpus = sum(1 for _, _, _, source in additions if source == "corpus")
    print(
        f"\n{len(additions)} entries ({len(added_words)} words) could be given a "
        f"gender: {from_corpus} from the corpus, "
        f"{len(additions) - from_corpus} from a suffix"
    )
    if objections:
        print(f"{len(objections)} of them igerman98 objects to, and they are skipped:")
        for word, found, why in objections[:20]:
            print(f"  {word:24} corpus says {''.join(sorted(found)):3} but {why}")

    if not (args.apply or args.add):
        print("\nnothing written; pass --apply or --add")
        return 0

    def rewrite(number: int, drop: set[str], add: set[str]) -> None:
        body, hash_, comment = lines[number].partition("#")
        word, flags = body.rstrip().split("/", 1)
        for gender in drop:
            flags = flags.replace(GENDER_FLAG[gender], "", 1)
        flags += "".join(GENDER_FLAG[g] for g in sorted(add))
        rebuilt = f"{word}/{flags}"
        lines[number] = f"{rebuilt} {hash_}{comment}" if hash_ else f"{rebuilt}\n"

    written = 0
    if args.apply:
        for number, _, recorded, found, _ in conflicts:
            rewrite(number, recorded, found)
            written += 1
    if args.add:
        for number, _, found, _ in additions:
            rewrite(number, set(), found)
            written += 1

    args.dictionary.write_text("".join(lines), encoding="utf-8")
    print(f"\nrewrote {written} entries in {args.dictionary}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
