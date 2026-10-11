#!/usr/bin/env python3
"""Record noun genders read off the gold morphology of a German UD treebank.

The article-cue oracle in `audit_german_gender.py` is exhausted on the prose
corpus and mostly says *not feminine* (`dem`, `des`, `einem`), which leaves
*Bereich*, *Umsatz* and *Verkauf* recorded masculine **and** neuter. A UD
treebank annotates the gender of every noun token, so it can name the one
gender instead.

Run it on the **train** splits. The test splits are what
`german_treebank_injection.py` measures against, and filling the dictionary
from them would grade the work with its own answers.

    python3 harper-core/src/language/german/scripts/add_german_treebank_genders.py \\
        UD_German-HDT/de_hdt-ud-train-*.conllu UD_German-GSD/de_gsd-ud-train.conllu

A lemma counts once at least `MIN_VOTES` singular tokens carry a gender and
`AGREEMENT` of them carry the same one. On the test splits, against the entries
that record a single gender, the treebank at that threshold agreed 99.2 % of the
time, and of the eleven disagreements nine were dictionary errors (*Sprung*,
*Schwung*, *Vorwand* recorded feminine, *Kupfer*, *Center* masculine) and two
were nouns with two genders (*Raster*, *Vorsitzende*).

Per lemma:

* a noun entry without a gender gets the treebank's;
* a noun entry with several genders, one of them the treebank's, is narrowed to
  it, unless `add_german_multi_gender_nouns.py` lists the word: those have two
  genders in standard German;
* a noun entry with a single, different gender is only reported. Corrections
  are hand-checked and go into `add_german_common_noun_genders.py`;
* a word the dictionary has only as a verb (*Abruf*, *Vorstand*, *Rahmen*)
  gets a noun line of its own, which the dictionary merges with the verb line.
  It carries no plural affix, so it builds no form. Not a neuter, though: that
  is the nominalized infinitive (*das Schreiben*), and a noun reading on every
  infinitive would reach *fragen*, *zahlen* and *regeln*, which are noun
  plurals too, and *sein*, which is a possessive. Nor a word with an adjective
  reading (*vorgesetzt*, the treebank's lemma for *Vorgesetzter*);
* a word in *-e* with an adjective reading is skipped: a nominalized adjective
  takes the gender of whoever it names (*der*/*die Vorsitzende*). So is an
  ordinal (*der*/*die*/*das Sechste*);
* a word with no line at all is skipped. Nearly all of those are compounds,
  which take their head's gender at lookup time.

Gender flags are plain properties (`M`, `F`, `Z`), so adding one builds no
surface form. Idempotent; the treebank itself is not stored.
"""

import collections
import importlib.util
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
DICT = HERE.parent / "dictionary.dict"

MIN_VOTES = 3
AGREEMENT = 0.9
FLAG = {"Masc": "M", "Fem": "F", "Neut": "Z"}
LINE = re.compile(r"^(?P<word>[^/\s]+)/(?P<head>~*)(?P<flags>\S*)(?P<rest>.*)$")
LEMMA = re.compile(r"[A-ZÄÖÜ][a-zäöüß]+")
NOUN_FLAGS = set("N0XYEMFZ")
# An ordinal used as a noun takes all three genders: *der*, *die*, *das Sechste*.
ORDINAL = re.compile(r"(erste|zweite|dritte|vierte|fünfte|sechste|siebte|achte|neunte|zehnte|elfte|zwölfte|zigste|ßigste)")


def multi_gender_words() -> set[str]:
    spec = importlib.util.spec_from_file_location("multi", HERE / "add_german_multi_gender_nouns.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return {word.lower() for word in module.GENDERS}


def treebank_genders(paths: list[str]) -> dict[str, str]:
    votes = collections.defaultdict(collections.Counter)
    for path in paths:
        for line in open(path, encoding="utf-8"):
            cols = line.rstrip("\n").split("\t")
            if len(cols) < 6 or not cols[0].isdigit() or cols[3] != "NOUN":
                continue
            lemma = cols[2]
            if not LEMMA.fullmatch(lemma):
                continue
            feats = dict(f.split("=", 1) for f in cols[5].split("|") if "=" in f)
            gender = feats.get("Gender", "")
            if gender in FLAG and feats.get("Number") == "Sing":
                votes[lemma.lower()][FLAG[gender]] += 1
    genders = {}
    for lemma, counter in votes.items():
        flag, count = counter.most_common(1)[0]
        if count >= MIN_VOTES and count >= AGREEMENT * sum(counter.values()):
            genders[lemma] = flag
    return genders


def is_noun_line(m: re.Match) -> bool:
    # `&` marks a plural form (`zwecke`), whose spelling is no singular lemma;
    # a `REPLACES` line only inflects a verb.
    flags = set(m["flags"])
    return bool(NOUN_FLAGS & flags) and "&" not in flags and "REPLACES" not in m["rest"]


def main(paths: list[str]) -> int:
    if not paths:
        print(__doc__, file=sys.stderr)
        return 2
    genders = treebank_genders(paths)
    multi = multi_gender_words()
    lines = DICT.read_text(encoding="utf-8").split("\n")

    by_word = collections.defaultdict(list)
    for i, line in enumerate(lines):
        m = LINE.match(line)
        if m:
            by_word[m["word"].lower()].append((i, m))

    filled = narrowed = appended_count = 0
    conflicts, appended = [], []
    for word, wanted in sorted(genders.items()):
        entries = by_word.get(word, [])
        # A nominalized adjective takes the gender of whoever it names: *der*
        # and *die Vorsitzende*, *Angestellte*, *Deutsche*. The treebank only
        # saw who happened to be meant.
        if word.endswith("e") and any("J" in m["flags"] for _, m in entries):
            continue
        if ORDINAL.fullmatch(word):
            continue
        nouns = [(i, m) for i, m in entries if is_noun_line(m)]
        recorded = set().union(*(set("MFZ") & set(m["flags"]) for _, m in nouns)) if nouns else set()

        if nouns and not recorded:
            for i, m in nouns:
                lines[i] = f"{m['word']}/{m['head']}{m['flags']}{wanted}{m['rest']}"
            filled += 1
        elif recorded == {wanted}:
            continue
        elif wanted in recorded and word not in multi:
            for i, m in nouns:
                flags = "".join(c for c in m["flags"] if c not in "MFZ" or c == wanted)
                if wanted not in flags:
                    flags += wanted
                lines[i] = f"{m['word']}/{m['head']}{flags}{m['rest']}"
            narrowed += 1
        elif recorded and wanted not in recorded:
            conflicts.append((word, "".join(sorted(recorded)), wanted))
        elif (
            entries
            and wanted != "Z"
            and all("V" in m["flags"] and "J" not in m["flags"] for _, m in entries)
        ):
            appended.append(f"{word}/~~Nh{wanted} # noun beside a verb line, gender from add_german_treebank_genders.py")
            appended_count += 1

    if appended:
        while lines and lines[-1] == "":
            lines.pop()
        lines += appended + [""]
    DICT.write_text("\n".join(lines), encoding="utf-8")

    print(
        f"{len(genders)} treebank genders: filled {filled}, narrowed {narrowed}, "
        f"noun lines added {appended_count}",
        file=sys.stderr,
    )
    for word, recorded, wanted in conflicts:
        print(f"CONFLICT {word}: dictionary {recorded}, treebank {wanted}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
