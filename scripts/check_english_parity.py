#!/usr/bin/env python3
"""Prove that adding languages leaves English exactly as it was.

The promise the language module system makes is that a build without any
language feature behaves like Harper did before the system existed, and that
turning languages on does not disturb English either. Both halves are checked
here by running the harper-core test suite twice and comparing every test that
exists in both runs:

    cargo test -p harper-core                          # English alone
    cargo test -p harper-core --features all-languages # English + every language

A test that passes in one run and fails in the other, or that disappears
between them, is a leak of language support into the English path.

Pass `--features <list>` to compare against a different second configuration,
e.g. `de` to check German alone.
"""

import argparse
import re
import subprocess
import sys

RESULT = re.compile(r"^test (?P<name>\S+) \.\.\. (?P<outcome>ok|FAILED|ignored)")


def run(features):
    """Map of test name -> outcome for one feature configuration."""
    command = ["cargo", "test", "-p", "harper-core"]
    if features:
        command += ["--features", features]
    command += ["--", "--test-threads", "4"]

    label = features or "no language features"
    print(f"running: {' '.join(command)}", flush=True)
    process = subprocess.run(command, capture_output=True, text=True)

    outcomes = {}
    for line in process.stdout.splitlines():
        match = RESULT.match(line)
        if match:
            outcomes[match["name"]] = match["outcome"]

    if not outcomes:
        print(process.stdout[-4000:], file=sys.stderr)
        print(process.stderr[-4000:], file=sys.stderr)
        raise SystemExit(f"no test results parsed for {label}")

    failed = sum(1 for o in outcomes.values() if o == "FAILED")
    print(f"  {label}: {len(outcomes)} tests, {failed} failed", flush=True)
    return outcomes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--features",
        default="all-languages",
        help="the second configuration to compare against (default: all-languages)",
    )
    args = parser.parse_args()

    english = run("")
    multilingual = run(args.features)

    problems = []

    for name, outcome in sorted(english.items()):
        if outcome == "FAILED":
            problems.append(f"fails without any language feature: {name}")

    shared = sorted(set(english) & set(multilingual))
    for name in shared:
        if english[name] != multilingual[name]:
            problems.append(
                f"{name}: {english[name]} without languages, "
                f"{multilingual[name]} with '{args.features}'"
            )

    vanished = sorted(set(english) - set(multilingual))
    for name in vanished:
        problems.append(f"only runs without languages, so it is never checked with them: {name}")

    if problems:
        print(f"\nEnglish behaviour differs between the two builds ({len(problems)}):")
        for problem in problems:
            print(f"  - {problem}")
        sys.exit(1)

    print(
        f"\nEnglish is identical in both builds: {len(shared)} shared tests agree, "
        f"{len(multilingual) - len(shared)} additional tests come with '{args.features}'."
    )


if __name__ == "__main__":
    main()
