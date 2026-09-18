#!/usr/bin/env python3
"""Stop the `-n` plural and the `-en` plural from being the same flag.

`Y` is documented as "Noun plural -n/-en" and it means it literally: every entry
carrying it gets **both** `word + "n"` and `word + "en"`. German nouns take one
or the other — `Diagnose` → `Diagnosen`, `Aufklärung` → `Aufklärungen` — so for
almost every one of the hundred thousand entries that carry `Y`, one of the two
is not a word. Many take neither, because their plural umlauts (`Arzt` →
`Ärzte`), doubles an `s` (`Ergebnis` → `Ergebnisse`) or does not exist
(`Chemie`).

Measured against the unmunched igerman98 list, of the 100842 `Y` entries it can
judge: 29065 take only `-n`, 23196 only `-en`, 31 both, and 48550 neither. That
is about 150000 generated non-words, every one a plausible typo of something.

So `-n` moves to its own flag and `Y` narrows to `-en`, and each entry is then
given whichever igerman98 says it takes:

    unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt
    scripts/split_german_plural_n.py --forms forms.txt
    scripts/split_german_plural_n.py --forms forms.txt --apply

An entry whose base igerman98 does not list at all is not judged — it keeps both
forms, so compounds igerman98 composes rather than lists (`skalierungstabelle`)
are unaffected. And as in `strip_german_noun_readings.py`, the whole dictionary
is expanded before and after: a flag stays wherever dropping it would lose a form
igerman98 does list.

This is the follow-up the headword import promised. `add_german_missing_words.py`
imports with the bare noun property and no affixes, on the grounds that
"`mirror_hunspell_flag.py` is the tool for adding one afterwards". This is that
step for the `-n`/`-en` plural.
"""

import argparse
import collections
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from mirror_hunspell_flag import forms_for, load_rule  # noqa: E402

DICT = pathlib.Path("harper-core/src/language/german/dictionary.dict")
ANNOTATIONS = pathlib.Path("harper-core/src/language/german/annotations.json")

PLURAL_EN = "Y"
PLURAL_N = "E"


def read_entries(path):
    out = []
    for index, line in enumerate(path.read_text(encoding="utf-8").splitlines()):
        entry, _, comment = line.partition("#")
        entry = entry.rstrip()
        if "/" not in entry:
            continue
        word, flags = entry.split("/", 1)
        out.append((index, word, flags, comment))
    return out


def load_affix_rules():
    data = json.loads(ANNOTATIONS.read_text(encoding="utf-8"))
    rules = {}
    for flag in data["affixes"]:
        try:
            rules[flag] = load_rule(flag)
        except Exception:
            continue
    return rules


def expand(entries, rules):
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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="write the changes")
    parser.add_argument("--forms", type=pathlib.Path, required=True)
    args = parser.parse_args()

    if not DICT.exists():
        print(f"{DICT} not found -- run from the repo root", file=sys.stderr)
        return 1

    rules = load_affix_rules()
    if PLURAL_N not in rules:
        print(
            f"'{PLURAL_N}' is not an affix yet. Define it in annotations.json as the "
            f"noun plural '-n' before running this.",
            file=sys.stderr,
        )
        return 1
    raw = args.forms.read_bytes()
    try:
        listed = raw.decode("utf-8").split()
    except UnicodeDecodeError:
        listed = raw.decode("iso-8859-1").split()
    # Harper's lookup is case-insensitive and a slice of the dictionary stores
    # capitalized nouns in lower case, so the oracle is consulted case-folded.
    known = {form.lower() for form in listed}

    entries = read_entries(DICT)
    by_index = {i: (i, w, f, c) for i, w, f, c in entries}
    before = expand(entries, rules)

    tally = collections.Counter()
    proposed = {}
    for index, word, flags, _ in entries:
        if PLURAL_EN not in flags:
            continue
        if word.lower() not in known:
            # Not something igerman98 can judge -- keep both forms exactly as they
            # are today by giving the entry the new flag alongside the old one.
            tally["unjudged"] += 1
            if PLURAL_N not in flags:
                proposed[index] = flags + PLURAL_N
            continue

        takes_n = (word + "n").lower() in known
        takes_en = (word + "en").lower() in known
        if takes_n and takes_en:
            tally["both"] += 1
            new = flags if PLURAL_N in flags else flags + PLURAL_N
        elif takes_n:
            tally["n"] += 1
            new = flags.replace(PLURAL_EN, PLURAL_N)
        elif takes_en:
            tally["en"] += 1
            new = flags
        else:
            tally["neither"] += 1
            new = flags.replace(PLURAL_EN, "")
        if new != flags:
            proposed[index] = new

    print(
        f"{tally['n']} entries take only -n, {tally['en']} only -en, {tally['both']} both, "
        f"{tally['neither']} neither; {tally['unjudged']} not judged"
    )

    def survivors(mapping):
        return expand([(i, w, mapping.get(i, f), c) for i, w, f, c in entries], rules)

    after = survivors(proposed)
    rounds = 0
    while True:
        lost = [f for f in set(before) - set(after) if f.lower() in known]
        if not lost:
            break
        blame = set()
        for form in lost:
            blame.update(i for i in before[form] if i in proposed)
        if not blame:
            print(f"{len(lost)} real words lost with nothing to blame -- aborting", file=sys.stderr)
            return 1
        for index in blame:
            del proposed[index]
        rounds += 1
        print(f"  round {rounds}: {len(blame)} entries left as they are to keep a real word")
        after = survivors(proposed)

    dropped = sorted(set(before) - set(after))
    print(f"{len(proposed)} entries rewritten, {len(dropped)} generated non-words disappear")
    for index in sorted(proposed)[:12]:
        _, word, flags, _ = by_index[index]
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
        lines[index] = rebuilt
    DICT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"rewrote {len(proposed)} entries in {DICT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
