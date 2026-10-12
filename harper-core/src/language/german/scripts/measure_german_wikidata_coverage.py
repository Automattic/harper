#!/usr/bin/env python3
"""How much of dictionary.dict does a permissively licensed source support?

    measure_german_wikidata_coverage.py DICT LEXICON.db OUTDIR TREEBANK.conllu...

LEXICON.db is the SQLite extract of Wikidata Lexemes (CC0) from
kenix/lexicon-packs, release lexicon-de-v3, `lexicon_de_grammar.db`
(SHA-256 of the gz 9393e9d5...3d055). Do not use that release's `_ipa` file,
which is CC BY-SA. The treebank (UD German-GSD train, CC BY-SA) is only read
for word frequencies; nothing from it is stored.

Each headword is classed as attested (word or its lemma is in the source),
attested via crude inflection stripping, attested via a rough compound split,
or unattested. The compound splitter here is an approximation, NOT Harper's
CompoundChecker; use the real checker for any final number.

Result with the trimmed pack (245,169 entries, GSD train, 222k tokens), open
classes only (prepositions, conjunctions, determiners and pronouns are not in
the pack and are hand-written instead):

    attested 84.0 % of tokens, via inflection 1.7 %, via compound 1.3 %,
    unattested 12.9 %  (adverbs/particles, proper names, dictionary noise)

Gender, capitalised nouns with an exact-case match and a gender on both sides:
1,445 of 1,479 identical, 20 overlapping, 14 conflicts (mostly -nis words, where
the source is right and the dictionary says neuter). It also supplies a gender
for 2,286 capitalised noun entries that have none.

Reads everything read-only; writes TSVs to OUTDIR.
"""
import sqlite3, sys, re, collections, pathlib, json

dict_path, db_path, out = sys.argv[1], sys.argv[2], pathlib.Path(sys.argv[3])
trees = sys.argv[4:]
out.mkdir(parents=True, exist_ok=True)

db = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
lemmas = {}          # casefold -> set of (lemma,pos,gender)
exact_lemma = set()
for l, p, g, f in db.execute("select lemma,pos,gender,forms from lemma"):
    lemmas.setdefault(l.casefold(), set()).add((l, p, g))
    exact_lemma.add(l)
forms = {}           # casefold form -> set of (lemma,pos)
for f, l, p in db.execute("select form,lemma,pos from inflected"):
    forms.setdefault(f.casefold(), set()).add((l, p))
# forms listed in lemma.forms (pl=..;pt=..;pp=..)
for l, p, f in db.execute("select lemma,pos,forms from lemma where forms is not null"):
    for part in re.split(r"[;|]", f):
        if "=" in part:
            part = part.split("=", 1)[1]
        for w in part.split("|"):
            if w:
                forms.setdefault(w.casefold(), set()).add((l, p))

for l,pp in db.execute("select distinct lemma,pos from inflected"):
    lemmas.setdefault(l.casefold(), set()).add((l, pp, None))
    exact_lemma.add(l)

def known_word(w):
    w = w.casefold()
    return w in lemmas or w in forms

# crude regular-inflection stripping: candidate lemmas of a surface form
SUF = ["", "e", "en", "n", "s", "es", "er", "em", "ern", "st", "est", "t", "et", "te", "ten", "tet", "ung", "innen", "in"]
def stems(w):
    w = w.casefold()
    yield w
    for s in ("en", "er", "es", "em", "e", "n", "s", "st", "t", "te", "ten", "et", "est", "innen"):
        if w.endswith(s) and len(w) - len(s) >= 3:
            b = w[: -len(s)]
            yield b
            yield b + "e"
            yield b + "en"
            yield b + "n"
            yield b + "in"
            yield b.translate(str.maketrans("äöü", "aou"))
            for u in ("ä", "ö", "ü"):
                pass
    if w.startswith("ge") and len(w) > 5:
        yield from stems(w[2:])

def via_inflection(w):
    for s in stems(w):
        if s in lemmas and s != w.casefold():
            return True
    return False

# compound split: every part a Wikidata lemma/form (>=3 chars), Fugen allowed
FUGEN = ["", "s", "es", "n", "en", "e", "er", "ens", "-"]
lemma_set = set(lemmas) | set(forms)
def split_compound(w, depth=0):
    w = w.casefold().replace("-", "")
    if w in lemma_set and depth > 0:
        return True
    if len(w) < 6 or depth > 3:
        return False
    for i in range(3, len(w) - 2):
        head, rest = w[:i], w[i:]
        for f in FUGEN:
            h = head[: -len(f)] if f and head.endswith(f) else None
            if f == "":
                h = head
            if h and len(h) >= 3 and h in lemma_set:
                if rest in lemma_set or split_compound(rest, depth + 1):
                    return True
    return False

# --- dictionary ---------------------------------------------------------
GEN = {"M": "m", "F": "f", "Z": "n"}
entries = []
for line in open(dict_path, encoding="utf-8"):
    line = line.rstrip("\n")
    if not line or line.startswith("#"):
        continue
    body = line.split(" #", 1)[0]
    if "/" in body:
        w, fl = body.split("/", 1)
    else:
        w, fl = body, ""
    entries.append((w, fl))
entries = entries[1:] if entries and entries[0][0].isdigit() else entries

def pos_of(fl):
    s = set(fl)
    if s & set("MFZN+"): return "noun"
    if "V" in s or "g" in s: return "verb"
    if "J" in s: return "adj"
    if s & set("PCDI45 6R r"):
        for k, n in (("P","prep"),("C","conj"),("D","det"),("4","det"),("I","pron"),("5","pron"),("6","conj"),("R","adv"),("r","adv")):
            if k in s: return n
    return "other"

# --- frequencies --------------------------------------------------------
form_n = collections.Counter(); lem_n = collections.Counter(); total = 0
lemma_known_tok = 0; form_known_tok = 0
tok_lemma_of = {}
for t in trees:
    for line in open(t, encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        c = line.split("\t")
        if "-" in c[0] or "." in c[0]:
            continue
        f, l, up = c[1], c[2], c[3]
        if up in ("PUNCT", "NUM", "SYM", "X"):
            continue
        total += 1
        form_n[f.casefold()] += 1
        lem_n[l.casefold()] += 1
        tok_lemma_of.setdefault((f, l, up), 0)
        tok_lemma_of[(f, l, up)] += 1

def freq(w):
    k = w.casefold()
    return max(form_n.get(k, 0), lem_n.get(k, 0))

# --- classify -----------------------------------------------------------
rows = []
for w, fl in entries:
    p = pos_of(fl)
    if "*" in fl:
        cls = "stem-marker"
    elif known_word(w):
        cls = "attested"
    elif via_inflection(w):
        cls = "attested-via-inflection"
    elif split_compound(w):
        cls = "attested-via-compound"
    else:
        cls = "unattested"
    rows.append((w, fl, p, cls, freq(w)))

with open(out / "entries.tsv", "w", encoding="utf-8") as fh:
    for r in rows:
        fh.write("\t".join(map(str, r)) + "\n")

by = collections.defaultdict(lambda: collections.Counter())
wt = collections.defaultdict(lambda: collections.Counter())
for w, fl, p, cls, fr in rows:
    by[p][cls] += 1; by["ALL"][cls] += 1
    wt[p][cls] += fr; wt["ALL"][cls] += fr
print(f"entries={len(rows)} treebank-tokens(no punct/num)={total}")
for p in ["ALL", "noun", "verb", "adj", "adv", "prep", "conj", "det", "pron", "other"]:
    n = sum(by[p].values()); wsum = sum(wt[p].values())
    print(f"\n{p}: {n} entries, weight {wsum}")
    for cls in ["attested", "attested-via-inflection", "attested-via-compound", "unattested", "stem-marker"]:
        print(f"   {cls:26s} {by[p][cls]:7d} {100*by[p][cls]/max(n,1):5.1f}%   freq {wt[p][cls]:8d} {100*wt[p][cls]/max(wsum,1):5.1f}%")

# token-level: does Wikidata know the treebank's gold lemma / form?
tk = collections.Counter()
for (f, l, up), n in tok_lemma_of.items():
    ok_l = l.casefold() in lemmas
    ok_f = known_word(f)
    tk[(up, "lemma" if ok_l else ("form" if ok_f else ("infl" if via_inflection(f) else ("comp" if split_compound(f) else "none"))))] += n
print("\nTOKEN-LEVEL coverage by gold UPOS (is gold lemma / form in Wikidata)")
ups = sorted({u for u, _ in tk}, key=lambda u: -sum(v for (a, b), v in tk.items() if a == u))
for u in ups:
    tot = sum(v for (a, b), v in tk.items() if a == u)
    parts = " ".join(f"{k}={100*tk[(u,k)]/tot:4.1f}%" for k in ["lemma", "form", "infl", "comp", "none"])
    print(f"  {u:6s} {tot:7d}  {parts}")
tot = sum(tk.values())
print("  ALL    ", tot, " ".join(f"{k}={100*sum(v for (a,b),v in tk.items() if b==k)/tot:4.1f}%" for k in ["lemma","form","infl","comp","none"]))

# --- genders ------------------------------------------------------------
agree = dis = onlyd = onlyw = 0
dis_rows = []
for w, fl, p, cls, fr in rows:
    if p != "noun": continue
    dg = {GEN[c] for c in fl if c in GEN}
    cands = lemmas.get(w.casefold())
    if not cands: continue
    wg = {g for (l, pp, g) in cands if pp == "n" and g}
    if dg and wg:
        if dg & wg and (dg == wg or True):
            agree += 1
            if dg != wg: dis_rows.append((w, "".join(sorted(dg)), "".join(sorted(wg)), fr, "partial"))
        else:
            dis += 1; dis_rows.append((w, "".join(sorted(dg)), "".join(sorted(wg)), fr, "conflict"))
    elif dg: onlyd += 1
    elif wg: onlyw += 1
print(f"\nGENDER: both have one: agree(overlap)={agree} conflict={dis}; only dict={onlyd}; only wikidata={onlyw}")
dis_rows.sort(key=lambda r: -r[3])
with open(out / "gender_disagree.tsv", "w", encoding="utf-8") as fh:
    for r in dis_rows: fh.write("\t".join(map(str, r)) + "\n")

un = sorted([r for r in rows if r[3] == "unattested"], key=lambda r: -r[4])
with open(out / "unattested_top.tsv", "w", encoding="utf-8") as fh:
    for r in un[:1000]: fh.write("\t".join(map(str, r)) + "\n")
