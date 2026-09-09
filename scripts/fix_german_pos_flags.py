#!/usr/bin/env python3
"""Repair German dictionary POS flags that corpus mining got wrong.

`GermanNounCapitalization` flags any word with an unambiguous noun reading
wherever it appears, and only treats a word as a noun/verb (or noun/adjective)
homograph -- context-gated -- when the entry *also* carries a verb or adjective
reading. A large slice of `dictionary.dict` was mechanically tagged `~~Nh` /
`~~NhY` / `~~NXh` / `~~NY` regardless of the real part of speech, so ordinary
verb forms and attributive adjectives get "corrected" mid-sentence.

This is the same class of bug as commits 2b8e10af1, 6f0ea43f6, 483a7f0cb and
556321ce9. It runs three high-precision passes:

  1. prefix_verb_forms  -- inseparable / directional-prefix finite verb forms
     (be-/ver-/ent-/zer-/emp-/er-/miss-/über-/unter-/durch-/wider-/hinter- + a
     finite ending, with <stem>+en a known infinitive) that are tagged noun-only
     -> ~~hV

  2. curated_verbs / curated_adverbs -- short hand-audited lists of strong-verb
     preterites and adverbs with no common noun homograph
     -> ~~hV  /  ~~r

  3. adjective bases -- a hand-audited list of very common adjectives whose base
     entry was noun/verb-only (`ganz/~rh`, `komplex/~NhY`, `modern/~~NhY`, ...)
     -> full ~~JOQRSTUWq declension. This restores the adjective reading of the
     base *and* of every declined form the affixes generate (`ganze`, `moderne`,
     ...), so those forms become proper noun/adjective homographs. The
     capitalization linter then keeps "die ganze Zeit" (attributive) apart from
     "das Ganze" (nominalised) by looking at the token to the right.

Bare "-e" forms are deliberately NOT stripped of their noun reading: "das Neue",
"das Wesentliche", "das Ganze" are real nouns.

Run from the repo root.  `--apply` writes; the default is a dry run.
Re-running is idempotent.
"""
import sys
import pathlib

DICT = pathlib.Path("harper-core/src/language/german/dictionary.dict")
NOUN = set("NMFZz")
VERB = set("Vjgtecxy")
ADJ = set("JqAOQRSTUW")
LOWER = "abcdefghijklmnopqrstuvwxyzäöüß"

NOUN_SUFFIX = ("heit", "keit", "schaft", "ung", "tät", "ität", "tion", "sion",
               "nis", "tum", "ling", "ismus", "ist", "ent", "ant", "and",
               "anz", "enz", "ur", "werk", "zeug", "nahme")
VERB_PREFIX = ("be", "ver", "ent", "zer", "emp", "er", "miss", "über", "unter",
               "durch", "wider", "wieder", "hinter")

CURATED_VERBS = set("""
sah saht aß aßt erfuhr erfuhrt empfing empfingt beschloss beschlosst
schlief schlieft fiel fielt bat batet las last gab gabt nahm nahmt kam kamt
ging gingt hielt hieltet dient dienst bleibt heißt läuft trägt fährt
schlägt wächst gräbt rät enthält behält beinhaltet umfasst besteht
prüfen prüft prüfe prüfst prüften prüfte
leeren leert leere leerst leerte leerten
erstellt erstelle erstellst erstellte erstellten
beherbergen beherbergt beherberge beherbergst beherbergte beherbergten
überwachen überwacht überwache überwachst überwachte überwachten
unterteilen unterteilt unterteile unterteilst unterteilte unterteilten
deckt decke deckst deckte deckten
klingt klingst
verließ verließt verließen
aussah aussahst aussaht aussahen
hält hältst
""".split())

CURATED_ADVERBS = set("überall sofort anders bereits vorab zeitweise".split())

# Very common adjectives whose base was noun/verb-only, so neither the base nor
# any of its declined forms had an adjective reading.
ADJ_BASE = set("ganz ander komplex besonder modern deutschsprachig weiter "
               "einzeln gross korrekt".split())


def parse(line):
    body = line.split("#", 1)[0].rstrip()
    if "/" not in body:
        return None
    w, fl = body.split("/", 1)
    return w, fl.strip().lstrip("~")


def main():
    apply = "--apply" in sys.argv
    lines = DICT.read_text(encoding="utf-8").splitlines()

    infin = set()
    for ln in lines:
        p = parse(ln)
        if p and ("j" in p[1] or "V" in p[1]) and p[0].endswith("en"):
            infin.add(p[0])

    def is_prefix_verb_form(w, fl):
        fs = set(fl)
        if not (fs & NOUN) or (fs & VERB) or (fs & ADJ):
            return False
        if any(w.endswith(s) for s in NOUN_SUFFIX):
            return False
        if not any(w.startswith(p) for p in VERB_PREFIX):
            return False
        for suf in ("test", "tet", "ten", "te", "st", "t"):
            if w.endswith(suf) and len(w) > len(suf) + 3:
                base = w[:-len(suf)]
                if (base + "en" in infin or base + "n" in infin
                        or base.rstrip("e") + "en" in infin):
                    return True
        return False

    counts = dict(prefix=0, verb=0, adverb=0, adj_base=0)
    out = []
    for ln in lines:
        p = parse(ln)
        if not p or not all(c in LOWER for c in p[0]):
            out.append(ln)
            continue
        w, fl = p
        fs = set(fl)
        keep_h = "h" if "h" in fl else ""

        if w in ADJ_BASE:
            out.append(f"{w}/~~JOQRSTUWq{keep_h} # retag: adjective, was noun/verb (corpus-mined)")
            counts["adj_base"] += 1
        elif w in CURATED_VERBS and (fs & NOUN) and not (fs & VERB):
            out.append(f"{w}/~~{keep_h}V # retag: verb form, was noun (corpus-mined)")
            counts["verb"] += 1
        elif w in CURATED_ADVERBS and (fs & NOUN) and not (fs & (VERB | ADJ)):
            out.append(f"{w}/~~r # retag: adverb, was noun (corpus-mined)")
            counts["adverb"] += 1
        elif is_prefix_verb_form(w, fl):
            out.append(f"{w}/~~{keep_h}V # retag: finite verb form, was noun (corpus-mined)")
            counts["prefix"] += 1
        else:
            out.append(ln)

    for k, v in counts.items():
        print(f"{k:10}: {v}")
    if apply:
        DICT.write_text("\n".join(out) + "\n", encoding="utf-8")
        print("written.")
    else:
        print("(dry run; pass --apply)")


if __name__ == "__main__":
    main()
