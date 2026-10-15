#!/usr/bin/env python3
"""Remove capitalized compounds from `dictionary.dict` that the compound checker reads the same way.

A compound such as `Sicherheitsorgan` needs no entry when `Sicherheit` and `Organ` are words:
`CompoundChecker` takes it apart at run time. An entry for it only costs startup time, because
the affix expansion writes it out with all its forms. But the checker is not a full replacement.
Measured on this dictionary, about half of the entries it could take over come back with other
properties: no number, no noun reading at all (`Abbaustoffen` read as an adjective), and now and
then another gender (`Kastration`). Rules that read those properties would change behaviour.

So the script asks the checker, word by word, and removes only what it answers identically:

    prune_german_compounds.py --framework harper-lang-test [--apply]

`harper-lang-test` (`just language-build`) loads a dictionary from a file, so the answer for the
current dictionary and for the dictionary without the candidates can be compared directly, with
no rebuild of `harper-core`. The comparison is `--dump-metadata`: the complete metadata of each
word, less `derived_from` (a compound has no lemma) and `orth_info` (how the spelling is cased).
A word the checker does not accept at all answers `-`, so a spelling difference shows up too.

Candidates are headwords that split into two other headwords, with an optional linking element
(`Sicherheit|s|Organ`), and that carry none of the flags other code reads from the word list
(`lexical_classes.rs`: numeral, abbreviation, foreign term, lower-case twin, strong present, plural
marker, plurale tantum), nor the name, proper-noun and stem-only marks. The split is a heuristic;
the metadata comparison is the test.

Removing a word can change how another one is read (it was a part of it), so the comparison is
repeated against the result until nothing more changes.

Only capitalized words are touched. Lower-case compounds are mostly separable-prefix verb forms,
which `can_head_a_lowercase_compound` accepts only when the whole verb exists.

Without `--apply` the dictionary is left alone and the counts are printed.
"""
import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dict_redundancy as dr  # noqa: E402

DICT = dr.DICT
ANNOTATIONS = DICT.parent / "annotations.json"
# Flags that carry information other code reads from the word list, or that mark an entry
# as something other than an ordinary common noun.
KEEP_FLAGS = set("123^%&+*!8")
# A short first part is a coincidence more often than a compound: Pro|gramm, Ar|beit.
dr.MIN_MOD = 4
MAX_ROUNDS = 4


def head(line):
    return line.partition("/")[0]


def candidates(original_lines, lowercase=False):
    es = [(head(l), l.partition("/")[2].split("#")[0].strip().lstrip("~")) for l in original_lines
          if l and not l.startswith("#")]
    known = {w.lower() for w, f in es if "!" not in f}
    return [w for w, f in es
            if w[0].isupper() != lowercase and not (KEEP_FLAGS & set(f)) and dr.split_compound(w, known)]


def dump(framework, dictionary, words_file):
    """`word<TAB>metadata` for every word of `words_file`, as the framework reads `dictionary`."""
    out = subprocess.run(
        [framework, "--language", "german", "--dict", str(dictionary),
         "--annotations", str(ANNOTATIONS), "--dump-metadata", str(words_file)],
        capture_output=True, text=True, check=True,
    ).stdout
    return {l.split("\t", 1)[0]: l.split("\t", 1)[1] for l in out.splitlines() if "\t" in l}


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--framework", required=True, help="path to the harper-lang-test binary")
    ap.add_argument("--apply", action="store_true", help="rewrite dictionary.dict")
    ap.add_argument("--lowercase", action="store_true",
                    help="consider lower-case entries (verbs with a prefix) instead of capitalized ones")
    a = ap.parse_args()

    original = DICT.read_text(encoding="utf8").split("\n")
    cands = candidates(original, a.lowercase)
    print(f"{len(cands)} candidates", file=sys.stderr)
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        words = tmp / "words.txt"
        words.write_text("\n".join(cands) + "\n", encoding="utf8")
        before = dump(a.framework, DICT, words)
        drop = set(cands)
        for round_ in range(1, MAX_ROUNDS + 1):
            trial = tmp / "trial.dict"
            trial.write_text("\n".join(l for l in original if head(l) not in drop) + "\n", encoding="utf8")
            after = dump(a.framework, trial, words)
            changed = {w for w in drop if before.get(w) != after.get(w)}
            print(f"round {round_}: {len(drop)} removed, {len(changed)} read differently", file=sys.stderr)
            if not changed:
                break
            drop -= changed
        else:
            sys.exit("the comparison did not settle; nothing written")
    print(f"{len(drop)} of {len(cands)} entries are redundant", file=sys.stderr)
    if a.apply:
        DICT.write_text("\n".join(l for l in original if head(l) not in drop), encoding="utf8")
        print(f"wrote {DICT}", file=sys.stderr)


if __name__ == "__main__":
    main()
