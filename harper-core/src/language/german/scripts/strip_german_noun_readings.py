#!/usr/bin/env python3
"""Take the noun reading off lower-case entries that are not nouns.

German capitalizes its nouns. A lower-case entry whose **capitalized** form
igerman98 does not list is therefore not a noun, whatever corpus mining tagged
it — `allenfalls`, `gleichwohl`, `wenngleich`, `mithin`, `desto`, `hierdurch`,
`diejenige` and tens of thousands of finite verb forms all carry `~~NhY` or
`~~NXh` and none of them is a noun.

`GermanNounCapitalization` flags any word with an unambiguous noun reading, so
each of these is a false positive waiting for the right sentence. On a corpus of
abstract German prose — the kind of text where they actually occur — they are
1250 of 1574 capitalization lints.

Two flags have to come off, not one. `N` is the noun property, but the
noun-plural affixes `X`, `Y`, `a`, `b` and `0` each carry a plural-noun reading
of their own, so removing `N` alone leaves the word a noun.

Those affixes also generate forms, and some of them are real words reached this
way and no other: `bedachte/~~YsV` builds `bedachten` through `Y`. So an affix is
removed only when every hunspell-known form it produces survives elsewhere in
the dictionary. The script expands the whole dictionary before and after to
check, exactly as `fix_german_double_declension.py` does.

    unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt
    harper-core/src/language/german/scripts/strip_german_noun_readings.py --forms forms.txt
    harper-core/src/language/german/scripts/strip_german_noun_readings.py --forms forms.txt --apply

Adding readings that are missing is the other half of this and lives in
`harper-core/src/language/german/scripts/fix_german_pos_flags.py`; this one only ever removes.
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

# The noun property, and the affixes that carry a plural-noun reading with them.
NOUN_PROPERTY = "N"
NOUN_AFFIXES = "XYab0"

# Where a noun affix has to stay because it is the only source of a real word,
# try handing the job to the verb affix that builds the same strings: `X` adds
# `-e` and `f` is the verb present `-e`, `Y` adds `-en`/`-n` and `j` is the verb
# present/plural `-en`. The swap is made only when the two generate exactly the
# same forms for that entry, so `geh/~~Xh` keeps `gehe` and stops calling it a
# plural noun.
VERB_SUBSTITUTE = {"X": "f", "Y": "j"}

# An entry with an adjective reading is left alone entirely. German nominalizes
# adjectives freely -- `das Gute`, `im Freien`, `für Deutsche` -- and those really
# are nouns, but igerman98's expanded list does not carry their capitalized
# spellings, so the oracle would call every one of them "not a noun". `deutsche`
# is the case that caught this: `Deutsche` is not in the form list at all.
ADJECTIVE_MARKERS = "JAqOQSTUW"

# The same applies one step removed: `wesentliche/~~NY` carries no adjective flag
# of its own, but `das Wesentliche` is a nominalization and `wesentlich` is an
# adjective. An entry that is a declined form of an adjective the dictionary has
# is therefore left alone too. Bare `-e` is the classic shape -- `das Gute`, `das
# Ganze`, `das Neue` -- but every ending can do it.
DECLENSION_ENDINGS = ("em", "en", "er", "es", "e")

# Verb affixes. An entry left with nothing but one of these and no property at
# all trips `scripts/validate_language_dict.py`, which reads a one-character flag
# string as a property name -- and it is right to complain: a word with an affix
# and no part of speech says nothing about itself. `rennt/~~NYG` becomes
# `rennt/~~VG`, which is what it always was.
VERB_AFFIXES = "cdefgijklmnsG"
VERB_PROPERTY = "V"


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


def with_a_part_of_speech(flags: str, properties: set) -> str | None:
    """Give back a flag string that still says what the word is.

    Stripping the noun reading can leave an entry holding affixes and no part of
    speech. Where those affixes are a verb's, the word is a verb and says so;
    where they are not, there is nothing to infer and the flags stand as they
    are.
    """
    body = flags.lstrip("~")
    if any(c in properties for c in body):
        return flags
    if body and all(c in VERB_AFFIXES for c in body):
        return flags + VERB_PROPERTY
    # Nothing to infer. Signal that by handing back the input unchanged, which
    # the caller reads as "leave this entry alone" -- better one entry keeping a
    # noun reading it should not have than an entry that claims to be nothing.
    return None


def load_affix_rules():
    """Every affix flag that generates forms, as `flag -> (kind, replacements)`."""
    data = json.loads(ANNOTATIONS.read_text(encoding="utf-8"))
    rules = {}
    for flag in data["affixes"]:
        try:
            rules[flag] = load_rule(flag)
        except Exception:  # a flag with no usable replacement list
            continue
    return rules


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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="write the changes")
    parser.add_argument(
        "--forms",
        type=pathlib.Path,
        required=True,
        help="the unmunched igerman98 form list; it is both the oracle and the guard",
    )
    args = parser.parse_args()

    if not DICT.exists():
        print(f"{DICT} not found -- run from the repo root", file=sys.stderr)
        return 1

    # `de_DE.aff` declares ISO8859-1 while the `.dic` beside it is usually the
    # UTF-8 frami variant, so decode optimistically and fall back.
    raw = args.forms.read_bytes()
    try:
        listed = raw.decode("utf-8").split()
    except UnicodeDecodeError:
        listed = raw.decode("iso-8859-1").split()
    known = set(listed)
    # Harper looks a word up case-insensitively, and a slice of `dictionary.dict`
    # stores capitalized nouns in lower case. A generated `verhältnissen` is what
    # makes `Verhältnissen` spell correctly, so the guard has to recognise it as
    # a real word even though igerman98 lists only the capitalized spelling.
    known_folded = {form.lower() for form in listed}

    def is_german(form: str) -> bool:
        return form in known or form.lower() in known_folded

    rules = load_affix_rules()
    properties = set(json.loads(ANNOTATIONS.read_text(encoding="utf-8"))["properties"])
    entries = read_entries(DICT)
    by_index = {i: (i, w, f, c) for i, w, f, c in entries}
    before = expand(entries, rules)

    adjective_bases = {
        word for _, word, flags, _ in entries if set(ADJECTIVE_MARKERS) & set(flags)
    }

    def could_be_nominalized(word: str) -> bool:
        return any(
            word.endswith(ending) and word[: -len(ending)] in adjective_bases
            for ending in DECLENSION_ENDINGS
        )

    proposed = {}
    for index, word, flags, _ in entries:
        if not word.islower():
            continue
        if not (set(NOUN_PROPERTY + NOUN_AFFIXES) & set(flags)):
            continue
        if set(ADJECTIVE_MARKERS) & set(flags) or could_be_nominalized(word):
            continue
        # igerman98 has to know the word at all, or there is no basis for
        # judging its part of speech. `university` is in the German dictionary as
        # a borrowing and in igerman98 in neither casing; without this the rule
        # reads "never capitalized, therefore not a noun" and strips it.
        if word not in known:
            continue
        # And the capitalized form must be absent: that is what says "not a noun".
        if word[:1].upper() + word[1:] in known:
            continue
        new = "".join(c for c in flags if c not in NOUN_PROPERTY + NOUN_AFFIXES)
        new = with_a_part_of_speech(new, properties)
        if new is not None and new != flags:
            proposed[index] = new

    print(f"{len(proposed)} lower-case entries carry a noun reading they should not")

    def survivors(mapping):
        return expand([(i, w, mapping.get(i, f), c) for i, w, f, c in entries], rules)

    # An affix may be the only thing producing a real word. Put back whatever the
    # dictionary would otherwise lose, and repeat until nothing real is at stake.
    after = survivors(proposed)
    swapped = {}
    rounds = 0
    while True:
        lost = [f for f in set(before) - set(after) if is_german(f)]
        if not lost:
            break
        blame = set()
        for form in lost:
            blame.update(i for i in before[form] if i in proposed)
        if not blame:
            print(f"{len(lost)} real words lost with nothing to blame -- aborting", file=sys.stderr)
            return 1
        for index in blame:
            _, word, flags, _ = by_index[index]
            # The bare `N` property never generates anything, so it can always
            # go. The affixes have to stay unless a verb affix builds the same
            # strings, in which case the reading moves with the job.
            kept = ""
            for flag in flags:
                if flag == NOUN_PROPERTY:
                    continue
                swap = VERB_SUBSTITUTE.get(flag)
                if swap and swap in rules and flag in rules:
                    if set(forms_for(word, *rules[flag])) == set(
                        forms_for(word, *rules[swap])
                    ):
                        kept += swap if swap not in flags else ""
                        swapped[index] = swapped.get(index, "") + flag
                        continue
                kept += flag
            if kept == flags:
                del proposed[index]
            else:
                proposed[index] = kept
        rounds += 1
        print(f"  round {rounds}: {len(blame)} entries keep their affixes, losing only '{NOUN_PROPERTY}'")
        after = survivors(proposed)

    dropped = sorted(set(before) - set(after))
    full = sum(1 for i, f in proposed.items() if not set(NOUN_AFFIXES) & set(by_index[i][2]) or
               not set(NOUN_AFFIXES) & set(f))
    print(f"{len(proposed)} entries rewritten, {len(dropped)} generated non-words disappear")
    if swapped:
        print(f"{len(swapped)} kept their affix but as the verb one that builds the same forms")
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
        rebuilt += f" #{comment}" if comment.strip() else " # not a noun: no capitalized form"
        lines[index] = rebuilt
    DICT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"rewrote {len(proposed)} entries in {DICT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
