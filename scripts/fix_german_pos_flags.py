#!/usr/bin/env python3
"""Repair German dictionary POS flags that corpus mining got wrong.

`GermanNounCapitalization` flags any word with an unambiguous noun reading
wherever it appears, and only treats a word as a noun/verb (or noun/adjective)
homograph -- context-gated -- when the entry *also* carries a verb or adjective
reading. A large slice of `dictionary.dict` was mechanically tagged `~~Nh` /
`~~NhY` / `~~NXh` / `~~NY` regardless of the real part of speech, so ordinary
verb forms and attributive adjectives get "corrected" mid-sentence.

This is the same class of bug as commits 2b8e10af1, 6f0ea43f6, 483a7f0cb and
556321ce9. It runs several high-precision passes, iterated to a fixed point
because they feed each other -- giving a mistagged infinitive its verb reading
reveals the finite forms built on it, and those reveal more participles.

The first three *replace* an entry's flags, which is only safe when the word has
no other reading at all:

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
     capitalization linter's noun-phrase chunker then keeps "die ganze Zeit"
     (attributive) apart from "das Ganze" (nominalised).

Bare "-e" forms are deliberately NOT stripped of their noun reading: "das Neue",
"das Wesentliche", "das Ganze" are real nouns.

The last pass (`add_missing_readings`) only ever *appends* a property flag, for
words that genuinely have two readings -- "die Vorsitzende" and "die vorsitzende
Richterin". It covers mistagged infinitives, present participles, comparatives
and hand-audited adjectives/adverbs. See the comment above it.

Run from the repo root.  `--apply` writes; the default is a dry run.
Re-running is idempotent.
"""
import re
import sys
import pathlib

DICT = pathlib.Path("harper-core/src/language/german/dictionary.dict")
# Words vouched for by LanguageTool on a German Wikipedia corpus: they appeared
# lower case in edited prose and LanguageTool saw nothing wrong, so whatever
# `dictionary.dict` says they are not unambiguous nouns. Regenerate with
# `.archive/german-language/scripts/derive_pos_fixes.py`.
POS_FIXES = pathlib.Path("scripts/german_pos_fixes.tsv")
NOUN = set("NMFZz")
VERB = set("Vjgtecxy")
ADJ = set("JqAOQRSTUW")
ADV = set("Rr")
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
verbracht verdrängt überführt erhält erfasst beläuft befolgt beeinflusst bedroht tun
""".split())

CURATED_ADVERBS = set("überall sofort anders bereits vorab zeitweise derzeit "
                      "höchstens ungefähr zufolge".split())

# Very common adjectives whose base was noun/verb-only, so neither the base nor
# any of its declined forms had an adjective reading.
ADJ_BASE = set("ganz ander komplex besonder modern deutschsprachig weiter "
               "einzeln gross korrekt unklar ungleich indirekt bewusst".split())


def parse(line):
    body = line.split("#", 1)[0].rstrip()
    if "/" not in body:
        return None
    w, fl = body.split("/", 1)
    return w, fl.strip().lstrip("~")


def load_pos_fixes():
    """word -> flag to append, from the LanguageTool-derived table."""
    flag_for = {"verb": "V", "adjective": "J", "adverb": "r"}
    fixes = {}
    if not POS_FIXES.exists():
        return fixes
    for line in POS_FIXES.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        word, pos = line.split("\t")[:2]
        if pos in flag_for:
            fixes[word] = (flag_for[pos], pos)
    return fixes


def main():
    apply = "--apply" in sys.argv
    lines = DICT.read_text(encoding="utf-8").splitlines()

    # The passes feed each other: giving a mistagged infinitive its verb reading
    # reveals the finite forms built on it, and those reveal more participles.
    # Iterate to a fixed point so one run is enough and re-running is a no-op.
    totals = {}
    for _ in range(10):
        lines, counts = run_passes(lines)
        for k, v in counts.items():
            totals[k] = totals.get(k, 0) + v
        if not any(counts.values()):
            break

    for k, v in totals.items():
        print(f"{k:12}: {v}")
    if apply:
        DICT.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print("written.")
    else:
        print("(dry run; pass --apply)")


def run_passes(lines):
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
            fixed = f"{w}/~~JOQRSTUWq{keep_h} # retag: adjective, was noun/verb (corpus-mined)"
            out.append(fixed)
            if fixed != ln:
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

    out, add_counts = add_missing_readings(out, infin)
    counts.update(add_counts)
    return out, counts


# --------------------------------------------------------------------------
# Additive pass
# --------------------------------------------------------------------------
# Everything above *replaces* an entry's flags, which is only safe for words
# with no other reading at all. The words below genuinely have two: "die
# Vorsitzende" and "die vorsitzende Richterin", "das Wochenende" and "die
# endende Frist". Their entries carry the noun reading but not the adjective /
# adverb / verb one, which makes GermanNounCapitalization treat them as
# *unambiguous* nouns and flag them wherever they appear.
#
# So this pass only ever **appends** a property flag. The word becomes a proper
# homograph and the noun-phrase chunker decides per occurrence. `J` (adjective),
# `r` (adverb) and `V` (verb) are property-only -- none of them is also an affix
# rule -- so nothing new is generated.

# Present participles. Only the bare "-end" / "-ende" forms are worth touching:
# "-endem/-enden/-ender/-endes" are already rejected by the linter's verb-shape
# test. The infinitive has to exist, which is what keeps Legende, Dividende,
# Wochenende, Torwartlegende and Tendenzwende out -- none of "torwartlegen",
# "dividen" or "wochenen" is a verb.
PARTICIPLE_SHAPE = re.compile(r"^([a-zäöüß]{3,}?)(end|ernd|elnd)e?$")
PARTICIPLE_INFINITIVE = {"end": "en", "ernd": "ern", "elnd": "eln"}


def is_present_participle(word, infinitives):
    m = PARTICIPLE_SHAPE.fullmatch(word)
    if not m:
        return False
    stem, suffix = m.group(1), m.group(2)
    return stem + PARTICIPLE_INFINITIVE[suffix] in infinitives

# Comparatives. The umlaut makes these hard to derive mechanically
# (gross -> groesser), so they are listed.
COMPARATIVES = set("""
größere spätere frühere ältere stärkere höhere geringere längere kürzere
kleinere neuere bessere jüngere niedrigere breitere engere tiefere weitere
schwächere schnellere langsamere einfachere schwierigere
""".split())

# Adjective *bases* with no adjective reading at all. These get the declension
# affixes as well, otherwise only the base is fixed and "mediale", "schärfere"
# and friends stay noun-only.
ADJ_EXTRA = set("""
medial urban nachhaltig scharf kontrovers innerdeutsch gesamtdeutsch
preisgünstig eigen bloß abstrakt effizient hilfreich real simultan mächtig
geheimnisvoll planvoll verbindlich intuitiv mental antik westlich nordöstlich
nationalsozialistisch handwerklich altgriechisch niederdeutsch althochdeutsch
""".split())

# Adverbs and particles.
ADV_EXTRA = set("""
demzufolge je anfangs oftmals mehrmals woanders irgendwie alleine darum vorne
zugrunde infrage gegebenenfalls annähernd höchstens halt
""".split())

# Finite verb forms. Words with a common noun homograph are deliberately absent
# -- macht/Macht, wacht/Wacht, würde/Würde, drang/Drang, halt/Halt.
VERB_EXTRA = set("""
erweist obliegt verweist regelt meint strebt lebt vorgibt handele einteilt
sprach floss galt stieß fällt zulässt müsse einnahm beansprucht erlebt
verdeutlicht besetzt angelegt eingesetzt eingestuft eingeteilt entlehnt
abgegrenzt abgefasst abgestreift aufgestaut trockengelegt durchgeführt erfüllt
erkämpft losgelöst eingedeicht abgedeicht ausgedehnt fördern fördert
""".split())


def verb_evidence(lines):
    """Words the dictionary itself says are verb forms, one way or another.

    - `<stem>/~~Vcej # REPLACES <infinitive>`: the infinitive was folded into a
      verb root, so the name in the comment is a verb form by construction.
    - a noun-only `-en`/`-ern`/`-eln` entry next to a sibling root that *does*
      carry a verb flag (`bohren/~~NMZ` beside `bohr/~~Vcej`) is the infinitive
      of that verb, not a plural noun.
    """
    roots, entries, replaced = set(), [], set()
    for ln in lines:
        p = parse(ln)
        if not p:
            continue
        entries.append(p)
        if set(p[1]) & VERB:
            roots.add(p[0])
        m = re.search(r"#.*\bREPLACES\s+([a-zäöüß]+)", ln)
        if m:
            replaced.add(m.group(1))

    infinitives = set(replaced)
    for w, fl in entries:
        if not all(c in LOWER for c in w):
            continue
        fs = set(fl)
        if not (fs & NOUN) or (fs & VERB) or (fs & ADJ):
            continue
        for suffix in ("en", "ern", "eln"):
            if w.endswith(suffix) and len(w) > len(suffix) + 2:
                stem = w[: -len(suffix)]
                root = stem if suffix == "en" else stem + suffix[:-1]
                if root in roots:
                    infinitives.add(w)
                break
    return infinitives


def add_missing_readings(lines, infinitives):
    counts = {"add_verb": 0, "add_adj": 0, "add_adv": 0}

    pos_fixes = load_pos_fixes()

    # Mistagged infinitives first: the participle rule below needs them.
    known_verbs = verb_evidence(lines) | VERB_EXTRA
    known_verbs |= {w for w, (_, pos) in pos_fixes.items() if pos == "verb"}
    staged = []
    for ln in lines:
        p = parse(ln)
        if p and all(c in LOWER for c in p[0]) and p[0] in known_verbs \
                and not (set(p[1]) & VERB):
            staged.append(rewrite(ln, p[0], p[1], "V", "verb"))
            counts["add_verb"] += 1
        else:
            staged.append(ln)

    infinitives = {w for w in known_verbs if w.endswith(("en", "ern", "eln"))}
    infinitives |= {
        p[0]
        for p in (parse(ln) for ln in staged)
        if p and p[0].endswith("en") and set(p[1]) & VERB
    }

    out = []
    for ln in staged:
        p = parse(ln)
        if not p or not all(c in LOWER for c in p[0]):
            out.append(ln)
            continue
        w, fl = p
        fs = set(fl)

        add, key, why = None, None, None
        if not (fs & ADJ) and (is_present_participle(w, infinitives)
                               or w in COMPARATIVES):
            add, key, why = "J", "add_adj", "adjective"
        elif not (fs & ADJ) and w in ADJ_EXTRA:
            add, key, why = "JqOQRSTUW", "add_adj", "adjective"
        elif not (fs & ADV) and w in ADV_EXTRA:
            add, key, why = "r", "add_adv", "adverb"
        elif w in pos_fixes:
            flag, pos = pos_fixes[w]
            missing = {"V": VERB, "J": ADJ, "r": ADV}[flag]
            if not (fs & missing):
                add = flag
                key = {"verb": "add_verb", "adjective": "add_adj",
                       "adverb": "add_adv"}[pos]
                why = pos

        if add is None:
            out.append(ln)
            continue

        out.append(rewrite(ln, w, fl, add, why))
        counts[key] += 1

    return out, counts


def rewrite(line, word, flags, add, why):
    """Append `add` to an entry's flags, keeping any existing comment."""
    comment = line.split("#", 1)[1].strip() if "#" in line else ""
    note = f"+{why} reading (was noun-only)"
    if comment and "reading (was noun-only)" not in comment:
        note = f"{comment}; {note}"
    elif comment:
        note = comment
    return f"{word}/~~{flags}{add} # {note}"


if __name__ == "__main__":
    main()
