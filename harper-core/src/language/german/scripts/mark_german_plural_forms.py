#!/usr/bin/env python3
"""Mark the noun plurals whose dative adds an `-n`.

German has one exceptionless noun ending left: the dative plural ends in
`-n`. *die Kinder* → *mit den Kindern*, *die Freunde* → *mit den Freunden*,
*die Bücher* → *in den Büchern*. Learners drop it constantly — *mit meinen
Freunde*, *seit drei Jahre*, *den Kinder gegeben* — and a rule can only
report that once it knows the form in front of it **is** a plural. The
dictionary's own number does not say so reliably: *Kinder* arrives as
singular and plural at once, merged from several entries, and *Lehrer*,
*Räuber*, *Reise* really are singulars that look the same.

hunspell's morphology does say so, in two ways:

    Freunde   st:Freund fl:E        the plural affix E (-e) on a singular
    Kinder    st:Kind   fl:R        the plural affix R (-er)
    Häuser    st:Haus   fl:p        the umlaut plural affix p
    Bücher    st:Bücher             an entry of its own, but *Buch* (umlaut
                                    and ending taken off) is a singular noun
                                    that takes the genitive -s/-es (flag S/T)

and keeps the singulars out: *Lehrer*, *Räuber* and *Spieler* are entries
with the genitive flag `S` themselves, and *Reise*, *Feder* have no umlaut to
take off. Each plural found is given the marker `&` in `dictionary.dict`,
which `spell::lexical_classes` reads back for `GermanDativePlural`.

    harper-core/src/language/german/scripts/mark_german_plural_forms.py \\
        --dic index.dic --aff index.aff --apply

The `.dic`/`.aff` are the UTF-8 igerman98 files wooorm/dictionaries ships.
`--apply` writes; the default is a dry run. Idempotent.
"""

import argparse
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from add_german_strong_imperatives import Hunspell  # noqa: E402

DICT = pathlib.Path(__file__).resolve().parent.parent / "dictionary.dict"
MARKER = "&"
LINE = re.compile(r"^(?P<word>[^/\s]+)/(?P<tilde>\*?~*)(?P<flags>\S*)(?P<rest>.*)$")
UMLAUTS = str.maketrans("äöü", "aou")


def umlaut_last_vowel(word: str) -> str:
    """*Haus* → *Häus*, *Baum* → *Bäum*, *Markt* → *Märkt*."""
    at = max(word.rfind("a"), word.rfind("o"), word.rfind("u"))
    if at < 0:
        return word
    if word[at] == "u" and at > 0 and word[at - 1] == "a":
        return word[: at - 1] + "äu" + word[at + 1:]
    return word[:at] + {"a": "ä", "o": "ö", "u": "ü"}[word[at]] + word[at + 1:]


def singular_nouns(entries: list[str]) -> dict[str, str]:
    """Capitalized headwords with a genitive flag (`S` -s, `T` -es): the
    masculine and neuter singulars. Mapped to their flags."""
    nouns = {}
    for line in entries:
        word, _, flags = line.partition("/")
        if word[:1].isupper() and ("S" in flags or "T" in flags):
            nouns[word] = flags
    return nouns


def plural_forms(hunspell: Hunspell, entries: list[str]) -> set[str]:
    singulars = singular_nouns(entries)
    plurals = set()
    for line in entries:
        word, _, flags = line.partition("/")
        if not word[:1].isupper() or word.endswith(("n", "s")):
            continue
        if word in singulars:
            continue
        # An umlaut plural with an entry of its own: *Bücher* ← *Buch*,
        # *Hände* ← *Hand*, *Brüder* ← *Bruder*.
        # Only the ending that is there comes off: *Kanüle* is no plural of
        # *Kanu*.
        plain = word.translate(UMLAUTS)
        bases = [plain]
        if plain.endswith("er"):
            bases.append(plain[:-2])
        elif plain.endswith("e"):
            bases.append(plain[:-1])
        # A base that builds its plural by affix has it there already, so
        # this entry is a different word: *Totschlag* → *Totschläge*, and
        # *Totschläger* is a person. `F` derives a feminine (*Schütze* →
        # *Schützin*), which only a singular does.
        if (
            plain != word
            and "F" not in flags
            and any(
                len(base) >= 3
                and base in singulars
                and not any(affix in singulars[base] for affix in "Rp")
                for base in bases
            )
        ):
            plurals.add(word)
    # The plural affixes on a singular: *Freunde*, *Jahre*, *Kinder*, *Bäume*.
    for base, flags in singulars.items():
        for affix in "ERp":
            if affix not in flags:
                continue
            for candidate in (f"{base}e", f"{base}er"):
                if any(a == f"st:{base} fl:{affix}" for a in hunspell.analyses(candidate)):
                    plurals.add(candidate)
        if "p" in flags:
            # The umlaut affix: *Haus* → *Häuser*, *Baum* → *Bäume*.
            stem = umlaut_last_vowel(base)
            for candidate in (f"{stem}e", f"{stem}er", stem):
                if f"st:{base} fl:p" in hunspell.analyses(candidate):
                    plurals.add(candidate)
    return {p for p in plurals if not p.endswith(("n", "s")) and hunspell.accepts(f"{p}n")}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--dic", type=pathlib.Path, required=True)
    parser.add_argument("--aff", type=pathlib.Path, required=True)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    hunspell = Hunspell(args.aff, args.dic)
    entries = args.dic.read_text(encoding="utf-8").splitlines()[1:]
    plurals = plural_forms(hunspell, entries)

    lines = DICT.read_text(encoding="utf-8").splitlines(keepends=True)
    index = {}
    for at, line in enumerate(lines):
        match = LINE.match(line.rstrip("\n"))
        if match:
            index.setdefault(match["word"], at)
    changed, added = 0, []
    for form in sorted(plurals):
        at = index.get(form)
        if at is None:
            added.append(
                f"{form}/~~{MARKER} # plural (mark_german_plural_forms.py)\n"
            )
            continue
        match = LINE.match(lines[at].rstrip("\n"))
        if MARKER in match["flags"]:
            continue
        lines[at] = f"{form}/{match['tilde']}{match['flags']}{MARKER}{match['rest']}\n"
        changed += 1
    print(f"{len(plurals)} plurals, {changed} entries marked, {len(added)} new")
    for form in sorted(plurals)[:60]:
        print("  ", form)
    if args.apply:
        DICT.write_text("".join(lines + added), encoding="utf-8")


if __name__ == "__main__":
    main()
