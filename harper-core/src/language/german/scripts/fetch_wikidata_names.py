#!/usr/bin/env python3
"""Fetch German proper names from Wikidata items and write `wikidata_names.tsv`.

    fetch_wikidata_names.py [--out harper-core/src/language/german/wikidata_names.tsv]

Wikidata items are CC0, so the result may be committed with its source named.
The queries run against the public Wikidata Query Service and ask for the German
label of

* given names (male, female, unisex),
* family names with at least two sitelinks,
* places with at least 1 000 inhabitants,
* countries, subdivisions, rivers, mountains, islands, seas and the like with at
  least three sitelinks,
* organisations, companies and clubs with at least ten,
* the demonyms of places (`Mannheimer`, `Irakerin`).

Labels of several words contribute each capitalised part of four letters or more
(`Sri Lanka` gives `Lanka`). Shorter names are typos as often as names. The
result is merged into the existing file, so a query the service refuses does
not lose what an earlier run found. The file has one `name<TAB>kind` per line.
"""

import argparse
import pathlib
import sys
import time
import http.client
import urllib.error
import urllib.parse
import urllib.request

ENDPOINT = "https://query.wikidata.org/sparql"
UA = "harper-german-dictionary/1.0 (https://github.com/Automattic/harper)"

QUERIES = {
    "given": 'SELECT DISTINCT ?l WHERE { VALUES ?c { wd:Q12308941 wd:Q11879590 wd:Q3409032 } '
             '?x wdt:P31 ?c ; rdfs:label ?l . FILTER(LANG(?l)="de") }',
    "family": 'SELECT DISTINCT ?l WHERE { ?x wdt:P31 wd:Q101352 ; wikibase:sitelinks ?s ; rdfs:label ?l . '
              'FILTER(LANG(?l)="de") FILTER(?s>=2) }',
    "place": 'SELECT DISTINCT ?l WHERE { ?x wdt:P1082 ?p ; rdfs:label ?l . FILTER(?p>=1000) FILTER(LANG(?l)="de") }',
    "place_dach": 'SELECT DISTINCT ?l WHERE { VALUES ?c { wd:Q515 wd:Q3957 wd:Q134626 wd:Q1548525 wd:Q1187811 wd:Q253019 wd:Q486972 wd:Q532 wd:Q1549591 wd:Q5119 wd:Q15284 wd:Q262166 wd:Q484170 wd:Q747074 wd:Q42744322 wd:Q123705 wd:Q1115575 wd:Q2514025 wd:Q3184121 wd:Q1093829 wd:Q15642541 wd:Q22865 wd:Q2039348 wd:Q1202578 wd:Q70208 wd:Q1707056 wd:Q2989398 wd:Q15617994 wd:Q56061 wd:Q200547 } VALUES ?k { wd:Q183 wd:Q40 wd:Q39 wd:Q347 } '
                  '?x wdt:P31 ?c ; wdt:P17 ?k ; rdfs:label ?l . FILTER(LANG(?l)="de") }',
    "place_world": 'SELECT DISTINCT ?l WHERE { VALUES ?c { wd:Q515 wd:Q3957 wd:Q134626 wd:Q1548525 wd:Q1187811 wd:Q253019 wd:Q486972 wd:Q532 wd:Q1549591 wd:Q5119 wd:Q15284 wd:Q262166 wd:Q484170 wd:Q747074 wd:Q42744322 wd:Q123705 wd:Q1115575 wd:Q2514025 wd:Q3184121 wd:Q1093829 wd:Q15642541 wd:Q22865 wd:Q2039348 wd:Q1202578 wd:Q70208 wd:Q1707056 wd:Q2989398 wd:Q15617994 wd:Q56061 wd:Q200547 } ?x wdt:P31 ?c ; wikibase:sitelinks ?s ; '
                   'rdfs:label ?l . FILTER(?s>=5) FILTER(LANG(?l)="de") }',
    "geo": 'SELECT DISTINCT ?l WHERE { VALUES ?c { wd:Q6256 wd:Q3624078 wd:Q10864048 wd:Q1221156 wd:Q35657 wd:Q5107 '
           'wd:Q46831 wd:Q4022 wd:Q8502 wd:Q23442 wd:Q165 wd:Q23397 wd:Q82794 wd:Q15324 wd:Q39816 wd:Q1620908 } '
           '?x wdt:P31 ?c ; wikibase:sitelinks ?s ; rdfs:label ?l . FILTER(?s>=3) FILTER(LANG(?l)="de") }',
    "demonym": 'SELECT DISTINCT ?l WHERE { ?x wdt:P1549 ?l . FILTER(LANG(?l)="de") }',
    "org": 'SELECT DISTINCT ?l WHERE { VALUES ?c { wd:Q4830453 wd:Q783794 wd:Q43229 wd:Q891723 wd:Q327333 wd:Q7278 '
           'wd:Q484652 } ?x wdt:P31 ?c ; wikibase:sitelinks ?s ; rdfs:label ?l . FILTER(?s>=10) FILTER(LANG(?l)="de") }',
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
        except (http.client.HTTPException, TimeoutError, ConnectionError) as err:
            if attempt == 5:
                raise urllib.error.URLError(str(err))
            print(f"{type(err).__name__}, waiting", file=sys.stderr)
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


def parts(label):
    """The words of a label that can stand alone as names."""
    return [w for w in label.split() if keep(w)]


def main():
    here = pathlib.Path(__file__).resolve().parent
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", default=str(here.parent / "wikidata_names.tsv"))
    ap.add_argument("--only", nargs="*", help="ask only for these kinds (the rest of the file is kept)")
    args = ap.parse_args()
    rows = {}
    old = pathlib.Path(args.out)
    if old.exists():
        for line in old.read_text(encoding="utf-8").splitlines():
            if line and not line.startswith("#"):
                name, _, kind = line.partition("\t")
                rows[name] = kind
    for kind, query in QUERIES.items():
        if args.only and kind not in args.only:
            continue
        try:
            got = run(query)
        except urllib.error.URLError as err:
            print(f"{kind}: skipped ({err})", file=sys.stderr)
            continue
        n = 0
        for label in got:
            for word in parts(label):
                rows.setdefault(word, kind)
                n += 1
        print(f"{kind}: {len(got)} labels, {n} kept", file=sys.stderr)
        time.sleep(20)
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
