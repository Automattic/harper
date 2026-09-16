#!/usr/bin/env python3
"""Measure whether Harper's German rules actually catch the mistakes they target.

The archived corpus is edited Wikipedia, so it measures one thing only: false
positives. Everything it can tell you is whether Harper stays quiet on correct
prose. It says nothing about whether a rule fires when it should, and a rule that
silently never fires looks perfect by that measure — which is exactly how the
dead `k`/`l`/`m`/`n` participle affixes survived so long.

This injects the mistakes German writers actually make into that same clean
prose, at known offsets, and reports what fraction of them Harper flags. Each
class is a real error, not a random perturbation:

    scripts/german_recall_check.py .archive/german-language/corpus

Recall alone is not the goal — a rule that flags everything would score 100% —
so read it next to `just language-lint-sources german`, which is the precision
side of the same coin.
"""

import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

CLI = Path("target/release/harper-cli")

# (name, pattern, replacement). The pattern matches correct German; the
# replacement is the mistake. `\b` keeps them off the inside of longer words.
INJECTIONS = [
    # Getrennt- und Zusammenschreibung
    ("garnicht", re.compile(r"\bgar nicht\b"), "garnicht"),
    ("wieviel", re.compile(r"\bwie viel\b"), "wieviel"),
    ("irgend etwas", re.compile(r"\birgendetwas\b"), "irgend etwas"),
    # Confused words
    ("seid/seit", re.compile(r"\bseit (Jahren|Jahrzehnten|Monaten|Wochen)\b"), r"seid \1"),
    ("wider/wieder", re.compile(r"\bwiderspr(icht|echen|ach)\b"), r"wiederspr\1"),
    ("wie/als", re.compile(r"\b(größer|kleiner|besser|älter|länger|höher) als\b"), r"\1 wie"),
    # Fixed misspellings that decompose into real words
    ("Standart", re.compile(r"\bStandard\b"), "Standart"),
    ("Diskusion", re.compile(r"\bDiskussion\b"), "Diskusion"),
    ("Vorraussetzung", re.compile(r"\bVoraussetzung\b"), "Vorraussetzung"),
    # Morphology the affix rules are responsible for
    ("missing epenthetic e", re.compile(r"\b(arbeit|red|öffn|rechn|arbeit)ete\b"), r"\1te"),
    ("wrong preterite", re.compile(r"\b(\w{3,})te\b"), r"\1ete"),
    # Capitalization
    ("lowercase noun", re.compile(r"\b(der|die|das) ([A-ZÄÖÜ])(\w{4,})\b"), None),
]


def apply_injection(text: str, pattern: re.Pattern, replacement) -> tuple[str, list]:
    """Rewrite every match, returning the new text and the mutated spans."""
    out = []
    spans = []
    last = 0
    for match in pattern.finditer(text):
        if replacement is None:
            # The capitalization case: lower-case the noun after the article.
            new = f"{match.group(1)} {match.group(2).lower()}{match.group(3)}"
        else:
            new = match.expand(replacement)
        if new == match.group(0):
            continue
        out.append(text[last : match.start()])
        start = sum(len(part) for part in out)
        out.append(new)
        spans.append((start, start + len(new)))
        last = match.end()
    out.append(text[last:])
    return "".join(out), spans


def lints_by_file(paths: list[Path]) -> dict:
    """Lint every path in one process.

    One invocation per file would dominate the runtime: each start-up expands the
    whole German dictionary, which costs more than the linting does.
    """
    if not paths:
        return {}
    result = subprocess.run(
        [str(CLI), "lint", "--dialect", "de", "--quiet", "--format", "json"]
        + [str(p) for p in paths],
        capture_output=True,
        text=True,
    )
    try:
        entries = json.loads(result.stdout)
    except json.JSONDecodeError:
        return {}
    return {
        entry["file"]: [
            (l["span"]["char_start"], l["span"]["char_end"]) for l in entry["lints"]
        ]
        for entry in entries
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("corpus", type=Path, help="directory of clean .md prose")
    parser.add_argument("--limit", type=int, default=60, help="files to sample")
    args = parser.parse_args()

    if not CLI.exists():
        print(f"{CLI} not found -- build it first", file=sys.stderr)
        return 1

    files = sorted(args.corpus.glob("*.md"))[: args.limit]
    if not files:
        print(f"no .md files in {args.corpus}", file=sys.stderr)
        return 1

    print(f"{len(files)} files\n")
    print(f"{'error class':24} {'injected':>9} {'caught':>7} {'recall':>8}")
    print("-" * 52)

    overall_injected = overall_caught = 0

    with tempfile.TemporaryDirectory() as tmp:
        for name, pattern, replacement in INJECTIONS:
            injected = caught = 0
            wanted = {}
            targets = []
            for source in files:
                text = source.read_text(encoding="utf-8")
                mutated, spans = apply_injection(text, pattern, replacement)
                if not spans:
                    continue
                target = Path(tmp) / source.name
                target.write_text(mutated, encoding="utf-8")
                wanted[source.name] = spans
                targets.append(target)

            flagged_by_file = lints_by_file(targets)
            for name_, spans in wanted.items():
                flagged = flagged_by_file.get(name_, [])
                injected += len(spans)
                for start, end in spans:
                    if any(fs < end and start < fe for fs, fe in flagged):
                        caught += 1
            if not injected:
                continue
            overall_injected += injected
            overall_caught += caught
            print(f"{name:24} {injected:9} {caught:7} {caught / injected:7.0%}")

    print("-" * 52)
    if overall_injected:
        print(f"{'total':24} {overall_injected:9} {overall_caught:7} "
              f"{overall_caught / overall_injected:7.0%}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
