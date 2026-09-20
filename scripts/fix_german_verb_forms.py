#!/usr/bin/env python3
"""Restore the conjugated forms German verbs are missing from the word list.

A German verb entry carries one flag per ending it can take: `f` for *ich
lerne*, `i` for *er lernt*, `j` for *wir lernen*, `d` for *er lernte*, `G` for
*du lernst*. An entry that carries some of them but not others has a hole in
its paradigm, and the form simply does not exist for Harper. That is why
`stattfindet`, `denkt` and `verbleibt` were reported as misspellings the moment
the compound decomposition stopped covering for them: `stattfinden/~~hc` has no
`i`, so it cannot form *stattfindet*.

**Which entries count as verbs is not obvious and getting it wrong is the whole
risk.** A filter that trusts `k`, `l`, `e` or `t` proposes `presseinformationt`
and `giftgrünt`, because `k` and `l` are also the n- and en-interfix compound
markers and `e`/`t` sit on noun and adjective entries. Even `V` is unreliable:
`abtragungen`, `sexpuppen` and `landwirtschaftlichen` all carry it. So the flag
filter only narrows the field, and the decision is made by an oracle:

    every form the ending would generate must be a form *of that entry*
    according to `hunspell -m`

That is what keeps `bären -> bärt` and `abtragungen -> abtragungt` out while
letting `ausscheiden -> ausscheidet` and `aufschreiben -> aufschreibt` in. The
`-t` form of a strong verb comes out as the second person plural (*ihr brecht
an*) rather than the third singular with its vowel change (*er bricht an*);
both are German, and the umlauted form is a separate gap this cannot fill.

    scripts/fix_german_verb_forms.py              # report only
    scripts/fix_german_verb_forms.py --apply      # write dictionary.dict

Needs `hunspell` and a **UTF-8** German dictionary. The one in `/usr/share` is
ISO-8859-1 and loses every umlaut; the script checks and refuses rather than
under-reporting by a fifth.
"""

import argparse
import collections
import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
DICT = ROOT / "harper-core/src/language/german/dictionary.dict"
ANNOTATIONS = ROOT / "harper-core/src/language/german/annotations.json"

# The endings this script can restore, in the order they are reported.
CONJUGATION = {
    "f": "ich lerne",
    "i": "er lernt",
    "j": "wir lernen",
    "d": "er lernte",
    "G": "du lernst",
}

# Flags that mean "verb" and cannot mean anything else. `V` is included even
# though it is over-applied; the oracle catches what it lets through.
VERB_FLAGS = set("Vjfcd")

# Flags that mean the entry is an inflected noun or adjective rather than an
# infinitive: a plural, or a declined adjective. The bare noun flags are *not*
# here on purpose — every German infinitive can be nominalized (`das Laufen`,
# `das Verbleiben`), so `N` or `Z` on an entry is no argument against it being
# a verb, and excluding them loses `verbleiben` and `überschreiten`.
NOMINAL_FLAGS = set("EXYab") | set("OQSTUW")

INFINITIVE = ("en", "ern", "eln")
# `abzubauen` has no finite forms at all. `zubauen` does, so the `zu` has to be
# preceded by something for the entry to be an extended infinitive.
ZU_INFINITIVE = re.compile(r".+zu[a-zäöüß]+en$")


def entries():
    """`(line index, word, flags, comment)` for every flagged dictionary line."""
    for index, line in enumerate(DICT.read_text(encoding="utf-8").splitlines()):
        body, _, comment = line.partition("#")
        body = body.strip()
        if not body or "/" not in body:
            continue
        word, _, flags = body.partition("/")
        yield index, word, flags, comment.strip()


def affix_rules(flag):
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
    """The character set the hunspell dictionary declares, from its `.aff`.

    This is not a detail. The German dictionary shipped in `/usr/share` is
    ISO-8859-1; handing it UTF-8 silently truncates every word at its first
    umlaut, so `überschreitet` arrives as `berschreitet` and the oracle says no
    to everything that matters. LanguageTool ships the same dictionary as
    UTF-8, which is why `--hunspell-dict` exists.
    """
    candidates = [pathlib.Path(f"{name}.aff")]
    candidates += [
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


def check_umlauts_survive(dictionary):
    """Refuse to run against a dictionary that mangles umlauts.

    The German dictionary in `/usr/share` declares ISO-8859-1, and on a UTF-8
    system hunspell truncates every word at its first umlaut whichever way the
    input is encoded: `überschreitet` arrives as `berschreitet`. Nothing errors
    — the oracle simply says no to every word that has an umlaut in it, which
    here is most of them, and the run silently under-reports by a fifth.
    """
    probe = "überschreitet"
    if probe in stems_of([probe], dictionary):
        return
    raise SystemExit(
        f"hunspell -d {dictionary} loses umlauts: '{probe}' is not analysed.\n"
        "Pass --hunspell-dict with a path to a UTF-8 copy of the dictionary, "
        "e.g. LanguageTool's:\n"
        "  .../resource/de/hunspell/de_DE"
    )


def stems_of(words, dictionary):
    """What `hunspell -m` says each form is a form *of*.

    Asking only whether hunspell knows the string is not enough: `dien` would
    get the `ich` ending because the form it builds, `die`, happens to be the
    article. The analysis says `st:die`, which is not the entry, so it is
    dropped. `entsteht` analyses as `st:entstehen`, which is.
    """
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
    return any(entry == stem or entry.endswith(stem) for stem in stems.get(form, ()))


def candidates(flag, rules):
    """Entries that should carry `flag` and do not, with the forms it adds."""
    out = []
    for index, word, flags, comment in entries():
        flagset = set(flags.lstrip("~"))
        if flag in flagset or not flagset & VERB_FLAGS or flagset & NOMINAL_FLAGS:
            continue
        if not word.endswith(INFINITIVE) or ZU_INFINITIVE.search(word):
            continue
        generated = forms(rules, word)
        if generated:
            out.append((index, word, flags, comment, generated))
    return out


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="write dictionary.dict")
    parser.add_argument(
        "--hunspell-dict",
        default="de_DE",
        help="hunspell dictionary to consult; pass a path to a UTF-8 copy if "
        "the system one is ISO-8859-1",
    )
    parser.add_argument(
        "--flags",
        # `j` is left out: the form it builds *is* the entry, so the check that
        # the form belongs to the entry proves nothing and `besen` and `boden`
        # come through as verbs.
        default="fidG",
        help="which endings to restore",
    )
    args = parser.parse_args()

    check_umlauts_survive(args.hunspell_dict)

    additions = collections.defaultdict(set)
    report = []

    for flag in args.flags:
        rules = affix_rules(flag)
        found = candidates(flag, rules)
        stems = stems_of(
            (f for _, _, _, _, gen in found for f in gen), args.hunspell_dict
        )

        # Every form the ending builds must be a form of *this* entry. A class
        # that generates two forms and gets one wrong would add the wrong one
        # too, and a form that is some other word entirely is not evidence.
        accepted = [c for c in found if all(belongs_to(c[1], f, stems) for f in c[4])]
        for index, _, _, _, _ in accepted:
            additions[index].add(flag)

        report.append((flag, CONJUGATION[flag], len(found), len(accepted)))
        if accepted:
            sample = ", ".join(f"{w} -> {g[0]}" for _, w, _, _, g in accepted[:4])
            print(f"  {flag} ({CONJUGATION[flag]}): {len(accepted)}/{len(found)}  e.g. {sample}")

    print(f"\n{'ending':22} {'missing':>9} {'confirmed':>10}")
    for flag, label, found, accepted in report:
        print(f"{flag} ({label}){'':6} {found:9} {accepted:10}")
    print(f"\n{len(additions)} entries to change")

    if not args.apply:
        print("dry run; pass --apply to write")
        return 0

    lines = DICT.read_text(encoding="utf-8").splitlines()
    for index, flags in additions.items():
        body, sep, comment = lines[index].partition("#")
        word, _, existing = body.rstrip().partition("/")
        lines[index] = f"{word}/{existing}{''.join(sorted(flags))}" + (
            f" {sep}{comment}" if sep else ""
        )
    DICT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {DICT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
