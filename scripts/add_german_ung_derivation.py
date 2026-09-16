#!/usr/bin/env python3
"""Give German verbs the `-ung` nominalization flags.

`Entscheidung`, `Bevölkerung` and `Veröffentlichung` were missing from the
dictionary outright, and not by accident: igerman98 does not store them either.
It stores the verb, `entscheiden`, with the suffix flag `J`, and derives the noun
and its plural from it. The only capitalized `Entscheidung*` entry in `de_DE.dic`
is `Entscheidungs/hij`, which is a compounding stem marked `NEEDAFFIX` — hunspell
rejects it as a word on its own.

Harper imported that stem as a standalone word and dropped the derivation, so the
whole `-ung` family went missing while `entscheidungs` stayed. This restores it:
`annotations.json` now mirrors hunspell's `SFX J` on flags `7` (singular) and `8`
(plural), and this script hands both flags to the verbs igerman98 marks with `J`.

The flags are digits because `annotations.json` keeps affixes and properties in
one namespace: a letter present in both tables is applied as both. `D`, the
obvious choice for `-ung`, is also the *determiner* property, and giving it to
these verbs turned every one of them into an article.

Membership comes from `de_DE.dic`, and every generated form is then checked
against the expanded form list, so a verb whose nominalization hunspell does not
actually accept is skipped.

Generate the form list once (it is large, and not committed):

    unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt

Run from the repo root:

    scripts/add_german_ung_derivation.py --forms forms.txt [--dic PATH] [--apply]

`--apply` writes; the default is a dry run. Re-running is idempotent.
"""

import argparse
import sys
from pathlib import Path

DICT = Path("harper-core/src/language/german/dictionary.dict")
DEFAULT_DIC = Path("/usr/share/hunspell/de_DE.dic")

# `7` derives the singular, `8` the plural. See `annotations.json`.
DERIVATION = "78"

# hunspell de_DE.aff, SFX J, as (strip, add, suffix the word must end in). The
# character class of the first rule is spelled out rather than parsed.
CLASS_ELN = tuple(c + "eln" for c in "bgkpßsz")
RULES = [
    ("n", "ung", CLASS_ELN),
    ("eln", "lung", ("eln",)),
    ("n", "ung", ("ern",)),
    ("en", "ung", ("en",)),
    ("el", "lung", ("el",)),
]


def generated_forms(word: str) -> list[str]:
    """Every `-ung` / `-ungen` form the `78` flags would produce for `word`."""
    forms = []
    for strip, add, endings in RULES:
        if not word.endswith(endings):
            continue
        stem = word[: len(word) - len(strip)] if strip else word
        forms.append(stem + add)
        forms.append(stem + add + "en")

    # The catch-all rule: anything not ending in `n`.
    if not word.endswith("n"):
        forms.append(word + "ung")
        forms.append(word + "ungen")

    return forms


def read_dic(dic: Path) -> str:
    """Decode `de_DE.dic`, whichever encoding it actually is.

    The shipped `de_DE.aff` declares `SET ISO8859-1`, but on this system the
    `.dic` is a symlink to the frami variant, which is UTF-8. Trusting the
    declaration silently mangles every umlaut and drops the verbs quietly — the
    script then reports no misses at all, because the entries never matched in
    the first place.
    """
    raw = dic.read_bytes()
    try:
        return raw.decode("utf-8")
    except UnicodeDecodeError:
        return raw.decode("iso-8859-1")


def lemmas_with_j(dic: Path) -> set[str]:
    """Every `de_DE.dic` headword carrying the `J` suffix flag."""
    lemmas = set()
    for line in read_dic(dic).splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "/" not in line:
            continue
        word, _, flags = line.partition("/")
        if "J" in flags:
            lemmas.add(word.lower())
    return lemmas


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
        default=DEFAULT_DIC,
        help=f"hunspell source dictionary (default: {DEFAULT_DIC})",
    )
    args = parser.parse_args()

    for path in (DICT, args.forms, args.dic):
        if not path.exists():
            print(f"{path} not found", file=sys.stderr)
            return 1

    forms = {
        line.strip().lower()
        for line in args.forms.read_text(encoding="utf-8", errors="replace").splitlines()
        if line.strip()
    }
    lemmas = lemmas_with_j(args.dic)
    print(f"{len(forms)} hunspell forms, {len(lemmas)} lemmas carrying J")

    out = []
    changed = 0
    skipped_unverified = 0
    samples = []

    for line in DICT.read_text(encoding="utf-8").splitlines(keepends=True):
        body, sep, comment = line.partition("#")
        stripped = body.strip()
        if not stripped or "/" not in stripped:
            out.append(line)
            continue

        word, _, flags = stripped.partition("/")
        plain = flags.replace("~", "")

        if word.lower() not in lemmas:
            out.append(line)
            continue

        missing = "".join(c for c in DERIVATION if c not in plain)
        if not missing:
            out.append(line)
            continue

        produced = generated_forms(word.lower())
        if not produced or not all(form in forms for form in produced):
            skipped_unverified += 1
            out.append(line)
            continue

        new_body = f"{word}/{flags}{missing}"
        trailing = body[len(body.rstrip()) :]
        out.append(f"{new_body}{trailing}{sep}{comment}")
        changed += 1
        if len(samples) < 15:
            shown = ", ".join(produced[:4])
            samples.append(f"{stripped}  ->  {new_body}   ({shown})")

    print(f"{changed} entries gain '{DERIVATION}'")
    print(f"{skipped_unverified} skipped: hunspell does not accept every generated form")
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
