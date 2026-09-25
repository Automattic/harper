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

    harper-core/src/language/german/scripts/german_recall_check.py .archive/german-language/corpus

With `--forms` it also measures the other kind of miss: plausible typos of the
words the corpus actually uses — a doubled letter, a dropped one from a pair, two
letters swapped, `ss`/`ß` — kept only when hunspell rejects them, so every one is
a real mistake. What Harper accepts there is a spell-check false negative.

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

# Fixed phrases whose nominalization *must* be capitalized, as whole phrases
# rather than a preposition crossed with a word list: `des Allgemeinen` and `im
# Weiteren` are not of this family, and injecting them scored the linter as
# having missed something that was never an error.
#
# `bei Weitem`, `ohne Weiteres`, `von Neuem`, `aufs Neue` and `zum Besten` are
# deliberately absent: the small letter is an equal variant in those, so
# lower-casing them produces correct German. See `german_fixed_nominalization.rs`.
FIXED_NOMINALIZATIONS = [
    ("im", w) for w in
    ("Übrigen", "Allgemeinen", "Wesentlichen", "Folgenden", "Besonderen", "Einzelnen",
     "Klaren", "Geringsten", "Nachhinein")
] + [("des", "Öfteren"), ("des", "Weiteren"), ("fürs", "Erste"), ("auf dem", "Laufenden")]

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
    ("als wie", re.compile(r"\b(größer|kleiner|besser|älter|länger|höher) als\b"), r"\1 als wie"),
    ("derselbe split", re.compile(r"\b(d)(er|ie|as)selbe\b"), r"\1\2 selbe"),
    ("in + year", re.compile(r"(?<![\w.])([12][0-9]{3}) (wurde|war|begann|erschien|folgte)\b"), r"in \1 \2"),
    # Fixed misspellings that decompose into real words
    ("Standart", re.compile(r"\bStandard\b"), "Standart"),
    ("Diskusion", re.compile(r"\bDiskussion\b"), "Diskusion"),
    ("Vorraussetzung", re.compile(r"\bVoraussetzung\b"), "Vorraussetzung"),
    # Morphology the affix rules are responsible for
    ("missing epenthetic e", re.compile(r"\b(arbeit|red|öffn|rechn|arbeit)ete\b"), r"\1te"),
    ("wrong preterite", re.compile(r"\b(\w{3,})te\b"), r"\1ete"),
    # Punctuation
    # The span has to include the word before the conjunction: that is where the
    # comma belongs, so that is what the linter marks.
    ("missing comma", re.compile(r"(\w+), (weil|obwohl|falls|sobald|nachdem|bevor|sofern) "), r"\1 \2 "),
    # Capitalization
    ("fixed nominalization",
     re.compile("|".join(rf"\b({p}) ({w})\b" for p, w in FIXED_NOMINALIZATIONS)),
     lambda m: f"{m.group(m.lastindex - 1)} {m.group(m.lastindex).lower()}"),
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
        elif callable(replacement):
            new = replacement(match)
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


def typo_variants(word: str) -> set[str]:
    """Plausible slips: a doubled letter, one dropped from a pair, a swap, ss/ß."""
    out = set()
    for i in range(len(word) - 1):
        if word[i] == word[i + 1]:
            out.add(word[:i] + word[i + 1 :])
        elif word[i] != word[i + 1]:
            out.add(word[:i] + word[i + 1] + word[i] + word[i + 2 :])
    for i, ch in enumerate(word):
        out.add(word[: i + 1] + ch + word[i + 1 :])
    if "ss" in word:
        out.add(word.replace("ss", "ß", 1))
    if "ß" in word:
        out.add(word.replace("ß", "ss", 1))
    return {m for m in out if m != word}


def report_typos(files: list[Path], forms_path: Path) -> None:
    """How many real typos of frequent corpus words does Harper let through?"""
    oracle = {
        line.strip()
        for line in forms_path.read_text(encoding="utf-8", errors="replace").splitlines()
        if line.strip()
    }

    frequency: dict[str, int] = {}
    for source in files:
        for word in re.findall(r"\b[A-Za-zÄÖÜäöüß]{5,}\b", source.read_text(encoding="utf-8")):
            frequency[word] = frequency.get(word, 0) + 1

    ranked = sorted(frequency.items(), key=lambda kv: -kv[1])[:3000]
    typos: dict[str, tuple[int, str]] = {}
    for word, count in ranked:
        if word not in oracle:
            continue
        for variant in typo_variants(word):
            if variant in oracle or variant[:1].upper() + variant[1:] in oracle:
                continue
            typos.setdefault(variant, (count, word))

    if not typos:
        return

    with tempfile.TemporaryDirectory() as tmp:
        target = Path(tmp) / "typos.md"
        target.write_text("\n\n".join(sorted(typos)), encoding="utf-8")
        result = subprocess.run(
            [str(CLI), "lint", "--dialect", "de", "--quiet", "--format", "json", str(target)],
            capture_output=True,
            text=True,
        )
        try:
            flagged = {
                l["matched_text"] for e in json.loads(result.stdout) for l in e["lints"]
            }
        except json.JSONDecodeError:
            flagged = set()

    missed = {t: v for t, v in typos.items() if t not in flagged}
    caught = len(typos) - len(missed)
    print(f"\n{'typos of frequent words':24} {len(typos):9} {caught:7} "
          f"{caught / len(typos):7.0%}")
    worst = sorted(missed.items(), key=lambda kv: -kv[1][0])[:10]
    if worst:
        print("  most frequent words whose typo slips through:")
        for variant, (count, word) in worst:
            print(f"    {word} -> {variant}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("corpus", type=Path, help="directory of clean .md prose")
    parser.add_argument("--limit", type=int, default=60, help="files to sample")
    parser.add_argument(
        "--forms",
        type=Path,
        help="hunspell form list from `unmunch`; enables the typo section",
    )
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

    if args.forms and args.forms.exists():
        report_typos(files, args.forms)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
