#!/usr/bin/env python3
"""Extract German lexemes from the Wikidata lexeme dump into a small SQLite file.

    extract_wikidata_lexemes.py latest-lexemes.json.gz german_lexemes.db

Wikidata lexemes are CC0, so the extract may be committed with its source named;
keep the downloaded dump itself (about 650 MB) outside the repository.

The dump is one JSON entity per line (inside a `[` ... `]` wrapper). Only
language Q188 (German) is kept. Tables:

    lexeme(id, lemma, category, gender, claims, senses)
                                          category is the Q-id's short name;
                                          gender: m, f, n, comma-joined when a lexeme has several;
                                          claims, senses: how many statements and senses the lexeme has,
                                          which tells a word in every dictionary from a one-off entry
    form(form, lexeme_id, features)       features: '|'-joined Q-ids
"""

import gzip
import json
import sqlite3
import sys

CATEGORY = {
    "Q1084": "noun",
    "Q24905": "verb",
    "Q34698": "adj",
    "Q380057": "adv",
    "Q4833830": "prep",
    "Q36484": "conj",
    "Q36224": "pron",
    "Q576271": "article",
    "Q184943": "contraction",
    "Q147276": "propn",
    "Q9788": "letter",
    "Q83034": "interj",
    "Q63116": "numeral",
    "Q184511": "particle",
    "Q62155": "affix",
}
GENDER = {"Q499327": "m", "Q1775415": "f", "Q1775461": "n"}


def main(src, dst):
    db = sqlite3.connect(dst)
    db.executescript(
        """
        drop table if exists lexeme; drop table if exists form;
        create table lexeme(id text primary key, lemma text, category text, gender text, claims integer, senses integer);
        create table form(form text, lexeme_id text, features text);
        """
    )
    n = kept = 0
    with gzip.open(src, "rt", encoding="utf-8") as fh:
        for line in fh:
            n += 1
            line = line.strip().rstrip(",")
            if len(line) < 3 or not line.startswith("{"):
                continue
            # cheap pre-filter before parsing the whole entity
            if '"language":"Q188"' not in line:
                continue
            e = json.loads(line)
            if e.get("language") != "Q188":
                continue
            lemma = (e.get("lemmas") or {}).get("de", {}).get("value")
            if not lemma:
                continue
            cat = CATEGORY.get(e.get("lexicalCategory"), e.get("lexicalCategory"))
            genders = set()
            for c in (e.get("claims") or {}).get("P5185", []):
                try:
                    g = GENDER.get(c["mainsnak"]["datavalue"]["value"]["id"])
                except KeyError:
                    continue
                if g:
                    genders.add(g)
            gender = ",".join(sorted(genders)) or None
            db.execute(
                "insert or replace into lexeme values (?,?,?,?,?,?)",
                (e["id"], lemma, cat, gender, len(e.get("claims") or {}), len(e.get("senses") or [])),
            )
            for f in e.get("forms", []):
                rep = (f.get("representations") or {}).get("de", {}).get("value")
                if rep:
                    db.execute(
                        "insert into form values (?,?,?)",
                        (rep, e["id"], "|".join(f.get("grammaticalFeatures", []))),
                    )
            kept += 1
            if kept % 20000 == 0:
                print(f"{n} lines read, {kept} German lexemes", file=sys.stderr)
    db.execute("create index if not exists ix_form on form(form)")
    db.execute("create index if not exists ix_lemma on lexeme(lemma)")
    db.commit()
    print(f"{n} lines read, {kept} German lexemes kept")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
