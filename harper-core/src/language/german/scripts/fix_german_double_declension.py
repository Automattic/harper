#!/usr/bin/env python3
"""Take the adjective declension flags off entries that are already declined.

`OQRST` are the five adjective declension endings, and they belong on the base
form: `klein/~~...OQRST` yields `kleine`, `kleinem`, `kleinen`, `kleiner`,
`kleines`. Several dozen entries are themselves one of those five forms and carry
the flags anyway, so they decline a second time and put `kleineree`,
`weißerem`, `vielee` and about two hundred more non-words into the dictionary.
Every one of them is a plausible typo of the real form, so they cost typo
detection for nothing.

Nine of those entries are worse: a comment lost its `#` and the text landed in
the flag string.

    höherer/~~Jq - comprtve jectveOQRST

`c`, `e`, `j`, `m`, `o`, `p`, `r`, `t` and `v` are all real flags, so `höherer`
— a comparative adjective — reads as a noun, a verb in the past and past
participle, and an adverb. Those entries are rewritten to `~~Jq`.

Most of what the doubled flags generate is not junk at all — `abstoßenderem` is
a real comparative, and `abstoßend` produces it too, through `U`. Dropping the
flags there changes nothing about the word set and only stops saying the same
thing twice. The script expands the whole dictionary before and after and keeps
an entry as it is whenever it would take a **hunspell-known** form with it, so
the only forms that disappear are ones igerman98 does not consider words.

    unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt
    harper-core/src/language/german/scripts/fix_german_double_declension.py --forms forms.txt
    harper-core/src/language/german/scripts/fix_german_double_declension.py --forms forms.txt --apply

Run `just language-lint-sources german` and `just language-recall german`
afterwards: this trades dictionary size for typo detection, and both move.
"""

import argparse
import collections
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from mirror_hunspell_flag import forms_for, load_rule  # noqa: E402

DICT = pathlib.Path("harper-core/src/language/german/dictionary.dict")

DECLENSION = "OQRST"
# The five endings those flags add. An entry ending in one of them is a candidate.
ENDINGS = ("em", "en", "er", "es", "e")

# `OQRST` are in the properties table as well as the affixes table, so taking
# them off also takes off a part of speech: `O`, `Q`, `S` and `T` each carry an
# adjective reading and `R` carries an adverb one. `J` and `r` say the same
# thing without generating anything, so the reading is moved rather than lost.
ADJECTIVE_AFFIXES = "OQST"
ADJECTIVE_PROPERTY = "J"
# Anything else that already supplies the reading, so it is not added twice.
ADJECTIVE_ELSEWHERE = "JAqUW"
ADVERB_AFFIX = "R"
ADVERB_PROPERTY = "r"

# The flag string the nine mangled comparatives should have had: adjective
# reading plus the compound-adjective marker, and no declension of their own.
MANGLED_MARKER = " - comprtve jectve"
MANGLED_FLAGS = "~~Jq"


def read_entries(path):
    """`[(line_index, word, flags, comment)]` for every entry, in file order."""
    out = []
    for index, line in enumerate(path.read_text(encoding="utf-8").splitlines()):
        entry, _, comment = line.partition("#")
        entry = entry.rstrip()
        if "/" not in entry:
            continue
        word, flags = entry.split("/", 1)
        out.append((index, word, flags, comment))
    return out


def expand(entries, rules):
    """Every surface form the dictionary produces, mapped to the entries making it."""
    produced = collections.defaultdict(set)
    for index, word, flags, _ in entries:
        produced[word].add(index)
        for flag in flags:
            rule = rules.get(flag)
            if rule is None:
                continue
            for form in forms_for(word, *rule):
                produced[form].add(index)
    return produced


def load_affix_rules():
    """Every affix flag that generates forms, as `flag -> (kind, replacements)`."""
    import json

    data = json.loads(
        pathlib.Path("harper-core/src/language/german/annotations.json").read_text(
            encoding="utf-8"
        )
    )
    rules = {}
    for flag in data["affixes"]:
        try:
            rules[flag] = load_rule(flag)
        except Exception:  # a flag with no usable replacement list
            continue
    return rules


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="write the changes")
    parser.add_argument(
        "--forms",
        type=pathlib.Path,
        required=True,
        help="the unmunched igerman98 form list; a form it lists is never given up",
    )
    args = parser.parse_args()

    if not DICT.exists():
        print(f"{DICT} not found -- run from the repo root", file=sys.stderr)
        return 1

    # `de_DE.aff` declares ISO8859-1 while the `.dic` beside it is usually the
    # UTF-8 frami variant, so decode optimistically and fall back.
    raw = args.forms.read_bytes()
    try:
        known_german = set(raw.decode("utf-8").split())
    except UnicodeDecodeError:
        known_german = set(raw.decode("iso-8859-1").split())

    rules = load_affix_rules()
    entries = read_entries(DICT)
    by_word = collections.defaultdict(set)
    for index, word, flags, _ in entries:
        by_word[word].update(flags)

    entries_by_index = {i: (i, w, f, c) for i, w, f, c in entries}

    before = expand(entries, rules)

    # Candidates: already a declined form, carrying declension flags, and with a
    # base that carries them too.
    proposed = {}
    for index, word, flags, comment in entries:
        mangled = MANGLED_MARKER in flags
        if not mangled:
            if not set(DECLENSION) & set(flags) or not word.endswith(ENDINGS):
                continue
            bases = [word[: -len(e)] for e in ENDINGS if word.endswith(e)]
            if not any(set(DECLENSION) & by_word.get(b, set()) for b in bases):
                continue
            new = "".join(c for c in flags if c not in DECLENSION)
            if set(ADJECTIVE_AFFIXES) & set(flags) and not set(ADJECTIVE_ELSEWHERE) & set(new):
                new += ADJECTIVE_PROPERTY
            if ADVERB_AFFIX in flags and ADVERB_PROPERTY not in new:
                new += ADVERB_PROPERTY
        else:
            new = MANGLED_FLAGS
        if new != flags:
            proposed[index] = new

    # Would a real German word become unreachable? Losing a form igerman98 does
    # not list is the point of the exercise; losing one it does list is a bug.
    def survivors(mapping):
        return expand([(i, w, mapping.get(i, f), c) for i, w, f, c in entries], rules)

    after = survivors(proposed)
    while True:
        lost_real = [f for f in set(before) - set(after) if f in known_german]
        if not lost_real:
            break
        blame = set()
        for form in lost_real:
            blame.update(i for i in before[form] if i in proposed)
        if not blame:
            print(f"{len(lost_real)} real words lost with nothing to blame -- aborting", file=sys.stderr)
            return 1
        for index in blame:
            del proposed[index]
        print(f"  kept {len(blame)} entries: they are the only source of a real word")
        after = survivors(proposed)

    dropped = sorted(set(before) - set(after))
    mangled = sum(1 for i in proposed if MANGLED_MARKER in entries_by_index[i][2])
    print(f"{len(proposed)} entries lose their own declension ({mangled} of them mangled)")
    print(f"{len(dropped)} generated non-words disappear, e.g. {dropped[:6]}")
    for index in sorted(proposed)[:12]:
        _, word, flags, _ = entries_by_index[index]
        print(f"    {word}/{flags}  ->  {word}/{proposed[index]}")

    if not args.apply:
        print("dry run; pass --apply to write")
        return 0

    lines = DICT.read_text(encoding="utf-8").splitlines()
    for index, word, flags, comment in entries:
        if index not in proposed:
            continue
        rebuilt = f"{word}/{proposed[index]}"
        if comment.strip():
            rebuilt += f" #{comment}"
        elif MANGLED_MARKER in flags:
            rebuilt += " # comparative adjective"
        lines[index] = rebuilt
    DICT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"rewrote {len(proposed)} entries in {DICT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
