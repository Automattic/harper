#!/usr/bin/env python3
"""Fetch German proper names from Wikidata items and write `wikidata_names.tsv`.

    fetch_wikidata_names.py [--out harper-core/src/language/german/wikidata_names.tsv]

Wikidata items are CC0, so the result may be committed with its source named.
The queries run against the public Wikidata Query Service and ask for the German
label of

* given names (male, female, unisex),
* family names with at least four sitelinks,
* places with at least 15 000 inhabitants,
* countries, first-level subdivisions, rivers, mountains, islands, seas and the
  like with at least six sitelinks,
* organisations, companies and clubs with at least fifteen.

Only one-word labels of four letters or more are kept (shorter ones are typos as often as names); the file has one `name<TAB>kind` per line.
"""

import argparse
import pathlib
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

ENDPOINT = "https://query.wikidata.org/sparql"
UA = "harper-german-dictionary/1.0 (https://github.com/Automattic/harper)"

QUERIES = {
    "given": 'SELECT DISTINCT ?l WHERE { VALUES ?c { wd:Q12308941 wd:Q11879590 wd:Q3409032 } '
             '?x wdt:P31 ?c ; rdfs:label ?l . FILTER(LANG(?l)="de") }',
    "family": 'SELECT DISTINCT ?l WHERE { ?x wdt:P31 wd:Q101352 ; wikibase:sitelinks ?s ; rdfs:label ?l . '
              'FILTER(LANG(?l)="de") FILTER(?s>=4) }',
    "place": 'SELECT DISTINCT ?l WHERE { ?x wdt:P1082 ?p ; rdfs:label ?l . FILTER(?p>=15000) FILTER(LANG(?l)="de") }',
    "geo": 'SELECT DISTINCT ?l WHERE { VALUES ?c { wd:Q6256 wd:Q3624078 wd:Q10864048 wd:Q1221156 wd:Q35657 wd:Q5107 '
           'wd:Q46831 wd:Q4022 wd:Q8502 wd:Q23442 wd:Q165 wd:Q23397 wd:Q82794 wd:Q15324 wd:Q39816 wd:Q1620908 } '
           '?x wdt:P31 ?c ; wikibase:sitelinks ?s ; rdfs:label ?l . FILTER(?s>=6) FILTER(LANG(?l)="de") }',
    "org": 'SELECT DISTINCT ?l WHERE { VALUES ?c { wd:Q4830453 wd:Q783794 wd:Q43229 wd:Q891723 wd:Q327333 wd:Q7278 '
           'wd:Q484652 } ?x wdt:P31 ?c ; wikibase:sitelinks ?s ; rdfs:label ?l . FILTER(?s>=15) FILTER(LANG(?l)="de") }',
}


def run(query):
    url = ENDPOINT + "?" + urllib.parse.urlencode({"query": query})
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "text/tab-separated-values"})
    for attempt in range(6):
        try:
            with urllib.request.urlopen(req, timeout=180) as resp:
                text = resp.read().decode("utf-8")
            break
        except urllib.error.HTTPError as err:
            # The service rate-limits and times out under load; wait and ask again.
            if err.code not in (429, 500, 502, 503, 504) or attempt == 5:
                raise
            print(f"HTTP {err.code}, waiting", file=sys.stderr)
            time.sleep(70)
    out = []
    for line in text.splitlines()[1:]:
        if line.startswith('"') and line.endswith('"@de'):
            out.append(line[1:-4])
    return out


def keep(label):
    return (
        len(label) >= 4
        and label[0].isupper()
        and all(c.isalpha() or c == "-" for c in label)
        and not label.endswith("-")
    )


def main():
    here = pathlib.Path(__file__).resolve().parent
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", default=str(here.parent / "wikidata_names.tsv"))
    args = ap.parse_args()
    rows = {}
    for kind, query in QUERIES.items():
        got = run(query)
        n = 0
        for label in got:
            if keep(label):
                rows.setdefault(label, kind)
                n += 1
        print(f"{kind}: {len(got)} labels, {n} kept", file=sys.stderr)
    header = (
        "# German proper names from Wikidata items (CC0, https://www.wikidata.org/).\n"
        "# Written by scripts/fetch_wikidata_names.py; one name and its kind per line.\n"
    )
    with open(args.out, "w", encoding="utf-8") as fh:
        fh.write(header)
        for label in sorted(rows, key=lambda w: (w.casefold(), w)):
            fh.write(f"{label}\t{rows[label]}\n")
    print(f"wrote {len(rows)} names to {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
