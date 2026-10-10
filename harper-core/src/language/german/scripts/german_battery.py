#!/usr/bin/env python3
"""Score Harper and LanguageTool on a per-class battery of German sentences.

A battery is a TSV of `class<TAB>err|ok<TAB>sentence`. Every `err` row is a
sentence that should be reported and every `ok` row one that should not, so
detection and false alarms are scored together — a class scored without its
correct sentences is not scored.

    docker start lt-bench
    cargo build --release --bin harper-cli --features de
    python3 harper-core/src/language/german/scripts/german_battery.py \
        harper-core/src/language/german/tests/batteries/grammar.tsv

Three things this gets right that a quick rewrite does not:

* **Harper is batched.** One file per sentence, one invocation. Loading the
  German dictionary costs about 1.5 s and the linting costs nothing, so a
  sentence-per-process loop takes minutes instead of seconds.
* **`--format compact` prints basenames**, not the paths it was given, so
  sentences are recovered from `NNN.md` rather than from the full path.
* **Typography is excluded by rule name, never by lint kind.** `Spaces` and
  `CommaFixes` would otherwise count as a hit on the right sentence for the
  wrong reason — but `GermanSubordinateComma` is `Punctuation`-kind too, and
  filtering that whole kind silently drops the comma class from its own score.
  The same holds on the LanguageTool side: its comma rules are `typographical`.
"""

import argparse
import collections
import json
import pathlib
import re
import shutil
import subprocess
import tempfile
import urllib.parse
import urllib.request

HARPER = "./target/release/harper-cli"
LANGUAGETOOL = "http://localhost:8010/v2/check"

IGNORE_HARPER = {"Spaces", "CommaFixes"}
IGNORE_LANGUAGETOOL = {"whitespace"}


def harper_hits(rows, work):
    """Which sentences Harper reports, as {index: [rule names]}."""
    for index, (_, _, text) in enumerate(rows):
        (work / f"{index:03d}.md").write_text(text + "\n")

    result = subprocess.run(
        [HARPER, "lint", "--dialect", "de", "--format", "compact"]
        + sorted(str(path) for path in work.glob("*.md")),
        capture_output=True,
        text=True,
    )

    hits = collections.defaultdict(list)
    for line in result.stdout.splitlines():
        match = re.match(r"^(\d{3})\.md:\d+:\d+: (\w+)::(\w+):", line)
        if match and match.group(3) not in IGNORE_HARPER:
            hits[int(match.group(1))].append(match.group(3))
    return hits


def languagetool_hits(text):
    """Which rules LanguageTool reports on one sentence."""
    data = urllib.parse.urlencode({"language": "de-DE", "text": text}).encode()
    with urllib.request.urlopen(LANGUAGETOOL, data) as response:
        return [
            match["rule"]["id"]
            for match in json.load(response)["matches"]
            if match["rule"]["issueType"] not in IGNORE_LANGUAGETOOL
        ]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("battery", type=pathlib.Path)
    parser.add_argument(
        "--per-sentence",
        action="store_true",
        help="list every sentence with what each tool said about it",
    )
    args = parser.parse_args()

    rows = [
        line.split("\t")
        for line in args.battery.read_text().splitlines()
        if line.strip() and not line.startswith("#")
    ]

    work = pathlib.Path(tempfile.mkdtemp(prefix="battery-"))
    try:
        hits = harper_hits(rows, work)
    finally:
        shutil.rmtree(work, ignore_errors=True)

    score = collections.defaultdict(collections.Counter)
    for index, (cls, kind, text) in enumerate(rows):
        harper = hits.get(index, [])
        languagetool = languagetool_hits(text)
        if args.per_sentence:
            print(
                f"{kind:3} {'H+' if harper else 'H-'} "
                f"{'LT+' if languagetool else 'LT-'}  {text[:60]:<61}"
                f"{','.join(harper)[:30]:<31}{','.join(languagetool)[:30]}"
            )
        for tool, found in (("harper", harper), ("lt", languagetool)):
            hit = "hit" if kind == "err" else "fp"
            total = "n" if kind == "err" else "ok"
            score[cls][f"{tool}_{hit}"] += 1 if found else 0
            score[cls][f"{tool}_{total}"] += 1

    if args.per_sentence:
        print()

    def cell(found, total):
        return f"{found}/{total}"

    def row(name, counts):
        print(
            f"{name:<26}{cell(counts['harper_hit'], counts['harper_n']):>8}"
            f"{cell(counts['lt_hit'], counts['lt_n']):>8}"
            f"{cell(counts['harper_fp'], counts['harper_ok']):>14}"
            f"{cell(counts['lt_fp'], counts['lt_ok']):>14}"
        )

    print(f"{'Klasse':<26}{'Harper':>8}{'LT':>8}{'H-Fehlalarm':>14}{'LT-Fehlalarm':>14}")
    total = collections.Counter()
    for cls in dict.fromkeys(row_[0] for row_ in rows):
        row(cls, score[cls])
        total.update(score[cls])
    print("-" * 70)
    row("GESAMT", total)


if __name__ == "__main__":
    main()
