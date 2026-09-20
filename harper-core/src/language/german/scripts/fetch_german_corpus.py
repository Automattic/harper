#!/usr/bin/env python3
"""Fetch German prose that is *about* something rather than about someone.

The archived corpus is 264 random German Wikipedia articles, which in practice
means biographies, places and events. Roughly 95% of what Harper still reports
there is a proper name neither it nor hunspell knows, and no rule will ever fix
that — it is a coverage question, and a dull one. The grammar never gets
exercised because the sentences are short and appositive.

This fetches the other kind of article: grammar, law, philosophy, mathematics,
medicine, economics. Abstract topics are written in long sentences with
subordinate clauses, nominalizations, participial attributes and passive voice —
the constructions the German linters actually model — and they name almost
nobody.

    scripts/fetch_german_corpus.py .archive/german-language/corpus-prose

Whole articles, not just the lead: `prop=extracts` with `explaintext`, so there
is no HTML to strip and no citation markers to clean. Wikipedia content is
CC BY-SA; the corpus is not committed (`.archive` is ignored), only this list.

`--limit` caps the fetch for a quick run. Re-running skips what is already there.

`--expand N` goes past the hand-written list. The 101 articles it produces are
*not* enough: measured as a saturation curve, the last ten files still each
contribute about four lint sites nobody had seen before, so the corpus is still
on the straight part of the curve and more of it still buys false positives.
Rather than guess at more titles, `--expand` asks Wikipedia which categories the
known-good topics live in and takes their other members, then filters by year
density — see `abstract`.
"""

import argparse
import pathlib
import random
import re
import sys
import time

import requests

API = "https://de.wikipedia.org/w/api.php"
UA = "harper-german-corpus/1.0 (https://github.com/Automattic/harper)"

# Topics chosen for prose density and name scarcity. Grouped only for reading.
TOPICS = [
    # Sprache und Grammatik
    "Deutsche Grammatik", "Nominalphrase", "Kasus", "Genitiv", "Dativ", "Akkusativ",
    "Konjunktiv", "Passiv", "Partizip", "Nebensatz", "Relativsatz", "Konjunktion (Wortart)",
    "Wortbildung", "Komposition (Grammatik)", "Derivation (Linguistik)", "Flexion",
    "Deklination (Grammatik)", "Konjugation (Grammatik)", "Wortart", "Satzglied",
    "Rechtschreibung", "Zeichensetzung", "Groß- und Kleinschreibung", "Silbentrennung",
    "Semantik", "Syntax", "Morphologie (Linguistik)", "Phonologie", "Pragmatik",
    "Sprachwandel", "Dialekt", "Standardsprache", "Fachsprache", "Lehnwort",
    # Recht und Verwaltung
    "Rechtsstaat", "Verwaltungsakt", "Vertragsrecht", "Schuldrecht", "Sachenrecht",
    "Verfassung", "Gewaltenteilung", "Grundrecht", "Strafrecht", "Zivilprozess",
    "Verjährung", "Haftung", "Eigentum", "Besitz", "Vollmacht", "Willenserklärung",
    # Philosophie
    "Erkenntnistheorie", "Ethik", "Ästhetik", "Metaphysik", "Logik", "Ontologie",
    "Erkenntnis", "Wahrheit", "Freiheit", "Gerechtigkeit", "Bewusstsein", "Vernunft",
    "Determinismus", "Idealismus", "Empirismus", "Rationalismus", "Phänomenologie",
    # Mathematik und Informatik
    "Wahrscheinlichkeit", "Stetigkeit", "Ableitung", "Integralrechnung", "Vektorraum",
    "Gruppe (Mathematik)", "Primzahl", "Mengenlehre", "Topologie (Mathematik)",
    "Algorithmus", "Datenstruktur", "Komplexitätstheorie", "Verschlüsselung",
    "Betriebssystem", "Datenbank", "Programmiersprache", "Künstliche Intelligenz",
    # Naturwissenschaft
    "Thermodynamik", "Quantenmechanik", "Relativitätstheorie", "Elektromagnetismus",
    "Chemische Bindung", "Katalyse", "Periodensystem", "Zellbiologie", "Evolution",
    "Genetik", "Photosynthese", "Ökosystem", "Klimawandel", "Plattentektonik",
    # Medizin und Psychologie
    "Immunsystem", "Stoffwechsel", "Blutkreislauf", "Nervensystem", "Diagnose",
    "Therapie", "Epidemiologie", "Impfung", "Wahrnehmung", "Gedächtnis", "Motivation",
    "Lernen", "Entwicklungspsychologie", "Persönlichkeit",
    # Wirtschaft und Gesellschaft
    "Marktwirtschaft", "Inflation", "Geldpolitik", "Arbeitsteilung", "Angebot und Nachfrage",
    "Steuer", "Sozialversicherung", "Bildungssystem", "Urbanisierung", "Migration",
    "Demokratie", "Föderalismus", "Bürokratie", "Öffentlichkeit", "Zivilgesellschaft",
    # Technik und Kunst
    "Statik", "Werkstoff", "Regelungstechnik", "Energieerzeugung", "Nachhaltigkeit",
    "Perspektive (Darstellung)", "Harmonielehre", "Kontrapunkt", "Erzähltheorie",
    "Metapher", "Rhetorik", "Typografie", "Architekturtheorie",
]


def slug(title: str) -> str:
    """A filename that survives a case-insensitive filesystem."""
    table = {"ä": "ae", "ö": "oe", "ü": "ue", "ß": "ss"}
    out = "".join(table.get(c, c) for c in title.lower())
    return re.sub(r"[^a-z0-9]+", "_", out).strip("_")


def fetch(titles: list[str], session: requests.Session) -> dict[str, str]:
    """Whole-article plain text, one title per request.

    `prop=extracts` batches up to 20 titles only for `exintro` extracts. Ask for
    the whole article and the limit is one — a batched request silently returns
    an extract for the first title and nothing for the rest, which looks like a
    run of missing articles rather than a mistake in the query.
    """
    out = {}
    for index, title in enumerate(titles, 1):
        try:
            response = session.get(
                API,
                params={
                    "action": "query",
                    "prop": "extracts",
                    "explaintext": "1",
                    "exsectionformat": "plain",
                    "titles": title,
                    "redirects": "1",
                    "format": "json",
                    "formatversion": "2",
                },
                timeout=60,
            )
            response.raise_for_status()
            for page in response.json().get("query", {}).get("pages", []):
                text = page.get("extract")
                if text:
                    out[page["title"]] = text
        except requests.RequestException as error:
            print(f"  {title}: {error}", file=sys.stderr)
        if index % 20 == 0 or index == len(titles):
            print(f"  fetched {index}/{len(titles)}", file=sys.stderr)
        time.sleep(0.3)
    return out


YEAR = re.compile(r"\b(1[0-9]{3}|20[0-2][0-9])\b")
# Prose articles carry a median 8 years per 1000 words, random ones 29. The
# threshold keeps ~90% of the hand-picked corpus and turns away ~70% of what a
# category walk drags in with it.
MAX_YEARS_PER_1000 = 20.0


def abstract(text: str) -> bool:
    """Is this about something rather than about someone?

    Dates are the cheapest tell there is. A biography, a battle or a football
    season is a chain of years; an article on the dative case or on catalysis
    has almost none. Counting them costs one regex and separates the two
    populations better than anything involving the text itself.
    """
    words = len(text.split())
    return bool(words) and len(YEAR.findall(text)) / words * 1000 <= MAX_YEARS_PER_1000


def categories_of(titles: list[str], session: requests.Session) -> list[str]:
    """The categories the known-good topics sit in, minus the housekeeping ones."""
    found: dict[str, int] = {}
    for start in range(0, len(titles), 20):
        response = session.get(
            API,
            params={
                "action": "query", "prop": "categories", "cllimit": "max",
                "clshow": "!hidden", "titles": "|".join(titles[start : start + 20]),
                "format": "json", "formatversion": "2",
            },
            timeout=60,
        )
        response.raise_for_status()
        for page in response.json().get("query", {}).get("pages", []):
            for category in page.get("categories", []):
                found[category["title"]] = found.get(category["title"], 0) + 1
        time.sleep(0.3)
    # A category two seed topics share is on-topic; one only a single topic has
    # is usually that topic's own container ("Kategorie:Immanuel Kant").
    return sorted(name for name, count in found.items() if count >= 2)


def members_of(categories: list[str], session: requests.Session) -> list[str]:
    out: list[str] = []
    for index, category in enumerate(categories, 1):
        response = session.get(
            API,
            params={
                "action": "query", "list": "categorymembers", "cmtitle": category,
                "cmnamespace": "0", "cmlimit": "max", "format": "json",
                "formatversion": "2",
            },
            timeout=60,
        )
        response.raise_for_status()
        out += [
            page["title"]
            for page in response.json().get("query", {}).get("categorymembers", [])
        ]
        print(f"  categories {index}/{len(categories)}", file=sys.stderr)
        time.sleep(0.3)
    # Shuffled, not sorted: a category's alphabetical head is stubs and list
    # articles, so taking the first N of a sorted list samples almost nothing
    # `usable` will accept. A fixed seed keeps the corpus reproducible.
    candidates = sorted(set(out))
    random.Random(0).shuffle(candidates)
    return candidates


def usable(text: str) -> bool:
    """Skip stubs and anything that is mostly a list rather than prose."""
    if len(text) < 2000:
        return False
    lines = [l for l in text.splitlines() if l.strip()]
    if not lines:
        return False
    # A list article has many short lines; prose has few.
    short = sum(1 for l in lines if len(l) < 60)
    return short / len(lines) < 0.5


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=pathlib.Path, help="directory to write into")
    parser.add_argument("--limit", type=int, help="fetch at most this many topics")
    parser.add_argument(
        "--expand", type=int, metavar="N",
        help="go beyond TOPICS: take N candidates from the categories the "
             "topics share, keeping only the ones that read as abstract prose",
    )
    args = parser.parse_args()

    args.output.mkdir(parents=True, exist_ok=True)
    session = requests.Session()
    session.headers["User-Agent"] = UA

    if args.expand:
        categories = categories_of(TOPICS, session)
        print(f"{len(categories)} shared categories", file=sys.stderr)
        candidates = members_of(categories, session)
        print(f"{len(candidates)} candidate articles", file=sys.stderr)
        wanted = [
            title for title in candidates
            if title not in TOPICS and not (args.output / f"{slug(title)}.md").exists()
        ][: args.expand]
    else:
        wanted = [t for t in TOPICS if not (args.output / f"{slug(t)}.md").exists()]
        if args.limit:
            wanted = wanted[: args.limit]
    if not wanted:
        print("nothing to fetch; every topic is already there")
        return 0

    print(f"fetching {len(wanted)} topics")
    articles = fetch(wanted, session)

    written = skipped = names = 0
    for title, text in articles.items():
        if not usable(text):
            skipped += 1
            continue
        if args.expand and not abstract(text):
            names += 1
            continue
        (args.output / f"{slug(title)}.md").write_text(text, encoding="utf-8")
        written += 1

    print(f"wrote {written}, skipped {skipped} as too short or list-shaped"
          + (f", {names} as too date-heavy to be abstract prose" if args.expand else ""))
    missing = sorted(set(wanted) - set(articles))
    if missing:
        print(f"{len(missing)} titles returned nothing: {missing[:8]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
