#!/usr/bin/env python3
"""Collect pinned GitHub READMEs and run Harper's passive rule for human review."""
import argparse
import base64
import hashlib
import json
from pathlib import Path
import random
import subprocess
import tempfile
from urllib.parse import quote


def api(endpoint):
    result = subprocess.run(["gh", "api", "--hostname", "github.com", endpoint], check=True, capture_output=True, text=True)
    return json.loads(result.stdout)


def save(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def collect(args):
    args.directory.mkdir(parents=True, exist_ok=False)
    candidates = api("search/repositories?q=" + quote(args.query) + "&sort=stars&order=desc&per_page=100")["items"]
    candidates = [repo for repo in candidates if not repo.get("private", False)]
    random.Random(args.seed).shuffle(candidates)
    manifest = {"query": args.query, "seed": args.seed, "documents": [], "failures": []}
    save(args.directory / "manifest.json", manifest)
    # A bounded sample of the first search page, rather than an unbiased sample of GitHub.
    for repo in candidates[:args.limit]:
        name = repo["full_name"]
        try:
            commit = api(f"repos/{name}/commits/{quote(repo['default_branch'], safe='')}")["sha"]
            readme = api(f"repos/{name}/readme?ref={commit}")
            if readme["size"] > 2_000_000 or readme.get("encoding") != "base64":
                raise ValueError("README exceeds the size limit or lacks base64 content")
            content = base64.b64decode(readme["content"], validate=False)
            content.decode("utf-8")  # Reject non-UTF-8 documents instead of silently replacing characters.
            filename = name.replace("/", "__") + ".md"
            (args.directory / filename).write_bytes(content)
            manifest["documents"].append({"repository": name, "commit": commit,
                "source": f"https://github.com/{name}/blob/{commit}/{quote(readme['path'], safe='/')}",
                "license": (repo.get("license") or {}).get("spdx_id"), "file": filename,
                "sha256": hashlib.sha256(content).hexdigest()})
        except (subprocess.CalledProcessError, ValueError, KeyError) as error:
            # Do not copy gh stderr: it may include account-specific diagnostics.
            manifest["failures"].append({"repository": name, "error": type(error).__name__})
        save(args.directory / "manifest.json", manifest)
    print(f"Collected {len(manifest['documents'])} documents; {len(manifest['failures'])} failures")


def review(args):
    manifest = json.loads((args.directory / "manifest.json").read_text(encoding="utf-8"))
    rows = []
    with tempfile.TemporaryDirectory(prefix="harper-corpus-dictionaries-") as dictionaries:
        for entry in manifest["documents"]:
            path = args.directory / entry["file"]
            if path.resolve().parent != args.directory.resolve():
                raise ValueError("Manifest file must be directly inside the corpus directory")
            data = path.read_bytes()
            if hashlib.sha256(data).hexdigest() != entry["sha256"]:
                raise ValueError(f"Corpus file changed: {entry['file']}")
            run = subprocess.run([str(args.harper.resolve()), "lint", str(path), "--only", "PassiveVoice",
                "--format", "json", "--quiet", "--no-parallel",
                "--user-dict-path", str(Path(dictionaries) / "user.txt"),
                "--file-dict-path", dictionaries], capture_output=True, text=True, timeout=300)
            # Harper exits nonzero when it finds lints. Valid JSON, rather than exit zero,
            # distinguishes those findings from a failed invocation.
            if run.returncode not in (0, 1):
                raise ValueError(f"Harper invocation failed for {entry['file']}")
            results = json.loads(run.stdout)
            if not isinstance(results, list) or len(results) != 1 or results[0].get("error"):
                raise ValueError(f"Unexpected Harper output for {entry['file']}")
            rows.append({"source": entry["source"], "sha256": entry["sha256"], "result": results[0]})

    save(args.output, {"purpose": "Unlabelled candidates for human review; not an accuracy score",
                       "documents": rows})
    print(f"Wrote results for {len(rows)} documents to {args.output}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    collect_parser = commands.add_parser("collect")
    collect_parser.add_argument("directory", type=Path)
    collect_parser.add_argument("--query", default="is:public stars:>=500 archived:false")
    collect_parser.add_argument("--seed", type=int, default=0)
    collect_parser.add_argument("--limit", type=int, choices=range(1, 101), default=100, metavar="1..100")
    collect_parser.set_defaults(action=collect)
    review_parser = commands.add_parser("review")
    review_parser.add_argument("directory", type=Path)
    review_parser.add_argument("--harper", type=Path, default=Path("target/debug/harper-cli"))
    review_parser.add_argument("--output", type=Path, required=True)
    review_parser.set_defaults(action=review)
    args = parser.parse_args()
    args.action(args)


if __name__ == "__main__":
    main()
