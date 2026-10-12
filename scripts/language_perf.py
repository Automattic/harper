#!/usr/bin/env python3
"""Time and peak memory of `harper-cli lint` for one language against English.

Three numbers per language, because they have different causes:
  start  a one-line file: the dictionary build, paid once per process
  lint   a ~200 KB sample minus start: the rules and spell suggestions
  peak   peak resident set of the whole process (ru_maxrss of the child)

Usage: language_perf.py <language> [--sample FILE] [--runs N]
"""
import argparse
import resource
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CLI = ROOT / "target/release/harper-cli"
TARGET_BYTES = 200_000
TINY = {"english": "Hello world.\n"}


def default_sample(language: str) -> str:
    """Text for `language`; fault-dense test sources unless the user passes --sample."""
    if language == "english":
        base = (ROOT / "harper-core/benches/essay.md").read_text(encoding="utf8")
    else:
        files = sorted((ROOT / f"harper-core/src/language/{language}/test_sources").glob("*.md"))
        if not files:
            sys.exit(f"no test_sources for {language}; pass --sample FILE")
        base = "\n".join(f.read_text(encoding="utf8") for f in files)
    return base * (TARGET_BYTES // max(len(base.encode()), 1) + 1)


def run(dialect: str, path: Path) -> tuple[float, int]:
    """(seconds, peak MB) for one lint of `path`."""
    code = (
        "import resource,subprocess,sys,time;t=time.time();"
        "r=subprocess.run(sys.argv[1:],capture_output=True,text=True);"
        "print(time.time()-t);"
        "print(resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss//1024)"
    )
    out = subprocess.run(
        [sys.executable, "-I", "-c", code, str(CLI), "lint", "-d", dialect, str(path)],
        capture_output=True, text=True, check=True,
    ).stdout.splitlines()
    return float(out[0]), int(out[1])


def measure(language: str, dialect: str, sample: str, runs: int, tmp: Path) -> dict:
    tiny, big = tmp / f"{language}_tiny.md", tmp / f"{language}_big.md"
    tiny.write_text(TINY.get(language, sample[:200].rsplit(" ", 1)[0] + "\n"), encoding="utf8")
    big.write_text(sample, encoding="utf8")
    start = statistics.median(run(dialect, tiny)[0] for _ in range(runs))
    full = [run(dialect, big) for _ in range(runs)]
    total = statistics.median(f[0] for f in full)
    return {"start": start, "lint": total - start, "peak": max(f[1] for f in full)}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("language")
    ap.add_argument("--sample", type=Path)
    ap.add_argument("--runs", type=int, default=3)
    a = ap.parse_args()
    if not CLI.exists():
        sys.exit(f"{CLI} missing; run `cargo build --release -p harper-cli`")
    sample_text = a.sample.read_text(encoding="utf8") if a.sample else None
    rows = []
    with tempfile.TemporaryDirectory() as t:
        for lang, dialect in [("english", "us"), (a.language, a.language)]:
            if lang == "english" and a.language == "english":
                continue
            text = sample_text if (sample_text and lang == a.language) else default_sample(lang)
            rows.append((lang, measure(lang, dialect, text, a.runs, Path(t))))
    print(f"{'':10} {'start':>8} {'lint':>8} {'peak':>8}   (median of {a.runs} runs)")
    for lang, m in rows:
        print(f"{lang:10} {m['start']:7.1f}s {m['lint']:7.1f}s {m['peak']:6d}MB")
    print("\nlint = 200 KB minus start. Fault-dense text inflates it (every unknown word"
          " triggers a suggestion search); compare like with like, or pass --sample.")


if __name__ == "__main__":
    main()
