#!/usr/bin/env python3
"""Fetch German prose in bulk, as counting material for the gender oracle.

This is **not** a measurement corpus and must not be used as one. The false
positive numbers in `german/README.md` are all against
`.archive/german-language/corpus-prose`, which is hand-picked for abstract
prose and stays fixed so those numbers stay comparable. Mixing the two would
silently change every baseline.

What this is for: `audit_german_gender.py` reads gender off unambiguous article
cues — *eine Frage*, *einen Baum*, *das Haus* — and needs three of them before
it will believe one. That test is 99.8 % precise and starved of text: 886
articles carry it to roughly half the noun entries. The text does not have to be
good, only German and plentiful, so this takes whatever Wikipedia hands out.

Two steps, because the API treats them differently. Titles come from
`generator=random` twenty at a time and cost almost nothing. Whole-article plain
text is limited to one title per request — ask for twenty and the response
carries an extract for the first and nothing for the rest, which reads like a
run of missing articles rather than a mistake in the query.

    harper-core/src/language/german/scripts/fetch_german_bulk.py \\
        .archive/german-language/corpus-bulk --articles 4000

Re-running skips what is already there, so it can be stopped and resumed.
"""
import argparse
import pathlib
import sys
import time

import requests

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from fetch_german_corpus import API, UA, fetch, slug, usable  # noqa: E402


def random_titles(wanted: int, session: requests.Session) -> list[str]:
    """Article titles from `generator=random`, twenty per request.

    The generator repeats itself, so this asks for more than it needs and
    de-duplicates; the loop gives up rather than spinning when a run of requests
    brings nothing new.
    """
    seen: dict[str, None] = {}
    idle = 0
    while len(seen) < wanted and idle < 20:
        before = len(seen)
        try:
            response = session.get(
                API,
                params={
                    "action": "query", "generator": "random", "grnnamespace": "0",
                    "grnlimit": "20", "format": "json", "formatversion": "2",
                },
                timeout=60,
            )
            response.raise_for_status()
            for page in response.json().get("query", {}).get("pages", []):
                seen.setdefault(page["title"], None)
        except requests.RequestException as error:
            print(f"  titles: {error}", file=sys.stderr)
        idle = idle + 1 if len(seen) == before else 0
        if len(seen) % 200 < 20:
            print(f"  {len(seen)}/{wanted} titles", file=sys.stderr)
        time.sleep(0.2)
    return list(seen)[:wanted]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=pathlib.Path, help="directory to write into")
    parser.add_argument("--articles", type=int, default=2000,
                        help="how many usable articles to aim for")
    parser.add_argument("--batch", type=int, default=500,
                        help="titles per round; a round is fetched before the next is asked for")
    args = parser.parse_args()

    args.output.mkdir(parents=True, exist_ok=True)
    session = requests.Session()
    session.headers["User-Agent"] = UA

    written = sum(1 for _ in args.output.glob("*.md"))
    print(f"{written} articles already there; aiming for {args.articles}")

    while written < args.articles:
        titles = random_titles(args.batch, session)
        titles = [t for t in titles if not (args.output / f"{slug(t)}.md").exists()]
        if not titles:
            print("the generator stopped producing new titles")
            break
        for title, text in fetch(titles, session).items():
            if not usable(text):
                continue
            (args.output / f"{slug(title)}.md").write_text(text, encoding="utf-8")
            written += 1
        print(f"{written}/{args.articles} usable articles")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
