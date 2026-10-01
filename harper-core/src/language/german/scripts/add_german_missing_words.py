#!/usr/bin/env python3
"""Import the igerman98 headwords Harper cannot reach at all.

Harper's German dictionary is a subset of igerman98's, and the compound checker
papers over much of the difference: half the missing headwords decompose into
parts Harper does have. The other half are simply absent — proper names
(`Caligula`, `Kalaschnikow`, `Rijswijk`), place-name derivations
(`Ihringshausener`) and ordinary vocabulary (`Pulsar`, `Styropor`, `Parataxe`,
`Lokativ`) — and every one of them is reported as a misspelling.

"Cannot reach" is asked of Harper itself rather than computed here, because the
compound decomposition is the thing being measured and a second implementation
of it would drift:

    cargo build --release -p harper-cli --features harper-core/multilingual
    harper-core/src/language/german/scripts/add_german_missing_words.py --apply

Entries are written with the bare noun property. No affix and no compound flag:
a freshly imported name has no vouched plural, and `harper-core/src/language/german/scripts/mirror_hunspell_flag.py`
is the tool for adding one afterwards, with every generated form checked against
the expanded form list.

Only capitalized headwords are imported. igerman98 also lists lower-case
compounding stems that hunspell itself rejects as words (`entscheidungs`), and
those must not become entries — see the `-ung` section of the language README.

`--apply` writes; the default is a dry run. Re-running is idempotent.
"""

import argparse
import json
import pathlib
import subprocess
import sys
import tempfile

DICT = pathlib.Path("harper-core/src/language/german/dictionary.dict")
CLI = pathlib.Path("target/release/harper-cli")
DEFAULT_DIC = pathlib.Path("/usr/share/hunspell/de_DE.dic")

# The noun property. Deliberately nothing else: see the module docstring.
NOUN_PROPERTY = "~~N"

# harper-cli holds the whole German dictionary in memory, so the words go in a
# few large files rather than one per call.
BATCH = 20000


def headwords(dic: pathlib.Path) -> list[str]:
    """Capitalized, purely alphabetic headwords of `de_DE.dic`.

    Decoding is deliberate: the shipped `de_DE.aff` declares `SET ISO8859-1`
    while the `.dic` beside it is usually the UTF-8 frami variant, and trusting
    the declaration mangles every umlaut.
    """
    raw = dic.read_bytes()
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError:
        text = raw.decode("iso-8859-1")

    out = []
    for line in text.splitlines()[1:]:
        word = line.strip().partition("/")[0].strip()
        if word and word.isalpha() and word[0].isupper():
            out.append(word)
    return out


def unknown_to_harper(words: list[str]) -> set[str]:
    """The subset `harper-cli` reports as a spelling error.

    One word per line, so each is its own paragraph: no noun-phrase context can
    make a word look known that is not. Only `GermanSpellCheck` is read back —
    a word on a line of its own also draws a sentence-capitalization lint, which
    says nothing about whether the word exists.
    """
    missing = set()
    for start in range(0, len(words), BATCH):
        chunk = words[start : start + BATCH]
        with tempfile.TemporaryDirectory() as tmp:
            target = pathlib.Path(tmp) / "words.md"
            target.write_text("\n\n".join(chunk), encoding="utf-8")
            result = subprocess.run(
                [str(CLI), "lint", "--dialect", "de", "--quiet", "--format", "json", str(target)],
                capture_output=True,
                text=True,
            )
        try:
            entries = json.loads(result.stdout)
        except json.JSONDecodeError:
            print("harper-cli produced no JSON; is it built?", file=sys.stderr)
            return set()
        for entry in entries:
            for lint in entry["lints"]:
                if lint["rule"] == "GermanSpellCheck":
                    missing.add(lint["matched_text"])
        print(f"    {start + len(chunk)}/{len(words)} checked, {len(missing)} missing")
    return missing


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="write the changes")
    parser.add_argument("--dic", type=pathlib.Path, default=DEFAULT_DIC)
    parser.add_argument(
        "--limit",
        type=int,
        help="import at most this many entries, the most frequent shapes first",
    )
    parser.add_argument(
        "--min-length",
        type=int,
        default=5,
        help=(
            "skip headwords shorter than this (default 5). Any entry of three "
            "characters or more becomes a compound element, and a short one is "
            "also a plausible typo of a frequent word: importing `Aa`, `Ahr`, "
            "`Alf` and `Abo` cost several hundred detections in "
            "`just language-recall german` for names almost nobody writes."
        ),
    )
    args = parser.parse_args()

    for path in (DICT, CLI, args.dic):
        if not path.exists():
            print(f"{path} not found -- run from the repo root", file=sys.stderr)
            return 1

    known = {
        line.split("/", 1)[0].strip()
        for line in DICT.read_text(encoding="utf-8").splitlines()
        if "/" in line
    }
    candidates = [
        word
        for word in headwords(args.dic)
        if word not in known and len(word) >= args.min_length
    ]
    print(f"{len(candidates)} capitalized headwords are not entries; asking harper-cli")

    missing = sorted(unknown_to_harper(candidates))
    if args.limit:
        missing = missing[: args.limit]

    print(f"{len(missing)} words Harper cannot reach even as compounds")
    for word in missing[:15]:
        print(f"    {word}/{NOUN_PROPERTY}")

    if not missing:
        return 0

    if args.apply:
        with DICT.open("a", encoding="utf-8") as handle:
            for word in missing:
                handle.write(f"{word}/{NOUN_PROPERTY} # igerman98 headword\n")
        print(f"appended {len(missing)} entries to {DICT}")
    else:
        print("dry run; pass --apply to write")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
