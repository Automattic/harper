#!/usr/bin/env python3
"""Shared machinery for repairing the German word list against hunspell.

Harper's German entries carry one flag per ending they may take. Thousands are
missing endings they should have, and the effect is not a missing niche form:
the word simply does not exist for Harper, and only the very permissive
compound decomposition keeps it from being reported. Both
`fix_german_verb_forms.py` and `fix_german_noun_forms.py` repair one class of
those flags, and both decide the same way:

    generate every form the ending would add, and keep the ending only if
    `hunspell -m` says each of those forms is a form *of that entry*

Asking only "does hunspell know this string?" is not enough. `dien` would take
the *ich* ending because the form it builds, `die`, happens to be the article,
and `bär` would take a plural because `bäre` is a form of *bären*. The `st:`
field of the morphological analysis names the entry a form belongs to, which is
the question actually being asked.

Needs a **UTF-8** German hunspell dictionary. The one in `/usr/share` declares
ISO-8859-1, and on a UTF-8 system hunspell truncates every word at its first
umlaut — `überschreitet` arrives as `berschreitet`. Nothing errors; the oracle
just says no to every word with an umlaut in it, which is most of the
interesting ones, and the run silently under-reports by a fifth. Call
[`check_umlauts_survive`] before using it.
"""

import argparse
import collections
import json
import pathlib
import re
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[5]
DICT = ROOT / "harper-core/src/language/german/dictionary.dict"
ANNOTATIONS = ROOT / "harper-core/src/language/german/annotations.json"


def entries(path=DICT):
    """`(line index, word, flags, comment)` for every flagged dictionary line."""
    for index, line in enumerate(path.read_text(encoding="utf-8").splitlines()):
        body, _, comment = line.partition("#")
        body = body.strip()
        if not body or "/" not in body:
            continue
        word, _, flags = body.partition("/")
        yield index, word, flags, comment.strip()


def flagset(flags):
    """The affix and property letters of a flag string, without the decoration."""
    return set(flags.replace("~", "").replace("*", ""))


def affix_rules(flag):
    """The `(condition, remove, add)` triples of one affix class."""
    data = json.loads(ANNOTATIONS.read_text(encoding="utf-8"))
    return [
        (re.compile(r["condition"] + "$"), r["remove"], r["add"])
        for r in data["affixes"][flag]["replacements"]
    ]


def forms(rules, word):
    """Every form the affix class builds from `word`."""
    out = []
    for condition, remove, add in rules:
        if not condition.search(word):
            continue
        if remove and not word.endswith(remove):
            continue
        out.append((word[: len(word) - len(remove)] if remove else word) + add)
    return out


def dictionary_encoding(name):
    """The character set the hunspell dictionary declares, from its `.aff`."""
    candidates = [pathlib.Path(f"{name}.aff")] + [
        pathlib.Path(directory) / f"{name}.aff"
        for directory in ("/usr/share/hunspell", "/usr/share/myspell")
    ]
    for path in candidates:
        if not path.exists():
            continue
        for line in path.read_text(encoding="latin-1").splitlines():
            if line.startswith("SET "):
                return line.split(None, 1)[1].strip().replace("ISO8859", "iso-8859")
    return "utf-8"


def analyse(words, dictionary):
    """`{form: {stem, ...}}` — what `hunspell -m` says each form is a form of."""
    words = sorted(set(words))
    if not words:
        return {}
    encoding = dictionary_encoding(dictionary)
    result = subprocess.run(
        ["hunspell", "-d", dictionary, "-m"],
        input=("\n".join(words) + "\n").encode(encoding, errors="replace"),
        capture_output=True,
        check=True,
    )
    out = collections.defaultdict(set)
    for line in result.stdout.decode(encoding, errors="replace").splitlines():
        if not line.strip():
            continue
        form, _, analysis = line.partition(" ")
        for field in analysis.split():
            if field.startswith("st:"):
                out[form].add(field[3:])
    return out


def belongs_to(entry, form, stems):
    """Is `form` a form of `entry` according to hunspell?

    The reported stem is the entry itself, or its unprefixed core: hunspell
    analyses `verbleibt` as `st:bleiben` with a `ver-` prefix flag, and
    `verbleiben` ends in `bleiben`. `dien` is not rescued by this, because
    `dien` does not end in `die`.
    """
    entry = entry.lower()
    return any(
        entry == stem.lower() or entry.endswith(stem.lower())
        for stem in stems.get(form, ())
    )


def check_umlauts_survive(dictionary):
    """Refuse to run against a dictionary that mangles umlauts."""
    probe = "überschreitet"
    if probe in analyse([probe], dictionary):
        return
    raise SystemExit(
        f"hunspell -d {dictionary} loses umlauts: '{probe}' is not analysed.\n"
        "Pass --hunspell-dict with a path to a UTF-8 copy of the dictionary, "
        "e.g. LanguageTool's:\n"
        "  .../resource/de/hunspell/de_DE"
    )


def argument_parser(description, default_flags):
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument("--apply", action="store_true", help="write dictionary.dict")
    parser.add_argument(
        "--hunspell-dict",
        default="de_DE",
        help="hunspell dictionary to consult; pass a path to a UTF-8 copy if "
        "the system one is ISO-8859-1",
    )
    parser.add_argument("--flags", default=default_flags, help="which endings to restore")
    parser.add_argument(
        "--matching",
        default=None,
        help="only consider entries whose headword matches this regular "
        "expression, e.g. 'nis$' for the nouns that double their s",
    )
    return parser


def entry_filter(pattern):
    """A predicate over headwords, from the `--matching` option."""
    if pattern is None:
        return lambda word: True
    compiled = re.compile(pattern)
    return lambda word: compiled.search(word) is not None


def apply_additions(additions):
    """Append the new flags to the lines that need them."""
    lines = DICT.read_text(encoding="utf-8").splitlines()
    for index, flags in additions.items():
        body, sep, comment = lines[index].partition("#")
        word, _, existing = body.rstrip().partition("/")
        lines[index] = f"{word}/{existing}{''.join(sorted(flags))}" + (
            f" {sep}{comment}" if sep else ""
        )
    DICT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return DICT
