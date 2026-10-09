#!/usr/bin/env python3
"""Add the forms of prefixed verbs Harper cannot reach.

`verbieten`, `verschwinden` and `versprechen` were in the dictionary as nouns
only (*das Verbieten*), so every finite form was a misspelling: *Sie
verschwindet*, *er verspricht es*, *das verbietet sich*, *er verschwand*. The
compound checker cannot help, because *ver-*, *zer-*, *ent-* are not words of
their own, and the strong forms (*verschwand*, *verspricht*) cannot be built
from the infinitive by the conjugation affixes anyway.

The forms are built from Harper's own word list: every lower-case word with one
of the inseparable prefixes in front. Hunspell decides which of those are
German, and Harper itself which of them it still rejects — the compound
decomposition is the thing being measured, so it is asked rather than
reimplemented:

    candidate = prefix + word        ver + spricht  -> verspricht
    hunspell accepts it              yes
    harper-cli reports it            yes            -> add it
    (or knows it only as a noun)

Each form is written as an entry of its own, with the verb property, or the
adjective property for a present participle (*verschwindend*) or an
adjective in *-lich* (*vertraulich*). No affix flags: the form is attested as
it stands, and nothing is generated from it.

    cargo build --release -p harper-cli --features harper-core/multilingual
    harper-core/src/language/german/scripts/add_german_prefixed_verb_forms.py \\
        --dic /usr/share/hunspell/de_DE.dic --aff /usr/share/hunspell/de_DE.aff --apply

The `.dic` must be the UTF-8 igerman98 one whose `.aff` says `SET UTF-8`
(wooorm/dictionaries ships it). `--apply` writes; the default is a dry run.
Re-running is idempotent.
"""

import argparse
import ctypes
import ctypes.util
import json
import pathlib
import re
import subprocess
import tempfile

DICT = pathlib.Path("harper-core/src/language/german/dictionary.dict")
CLI = pathlib.Path("target/release/harper-cli")

PREFIXES = ["ver", "zer", "ent", "emp", "er", "be", "miss", "über", "unter", "hinter", "wider"]

# A present participle and its declined forms, or an adjective in -lich/-bar.
ADJECTIVE = re.compile(r"(end|lich|bar|wert)(e|em|en|er|es)?$|lich(er|st)(e|em|en|er|es)?$")

COMMENT = "# prefixed form, hunspell-attested (add_german_prefixed_verb_forms.py)"


class Hunspell:
    def __init__(self, aff: pathlib.Path, dic: pathlib.Path):
        lib = ctypes.CDLL(ctypes.util.find_library("hunspell-1.7") or "libhunspell-1.7.so.0")
        lib.Hunspell_create.restype = ctypes.c_void_p
        lib.Hunspell_create.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        lib.Hunspell_spell.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        self.lib = lib
        self.handle = lib.Hunspell_create(str(aff).encode(), str(dic).encode())

    def accepts(self, word: str) -> bool:
        return self.lib.Hunspell_spell(self.handle, word.encode("utf-8")) != 0


def harper_words() -> set[str]:
    out = subprocess.run(
        [str(CLI), "words", "--dialect", "de"], capture_output=True, text=True, check=True
    ).stdout
    return {line.strip().strip('"') for line in out.splitlines()}


def rejected_by_harper(words: list[str]) -> set[str]:
    """The subset `harper-cli` reports as a spelling error, one word per paragraph."""
    with tempfile.TemporaryDirectory() as tmp:
        path = pathlib.Path(tmp) / "words.md"
        path.write_text("".join(f"{w}\n\n" for w in words), encoding="utf-8")
        out = subprocess.run(
            [str(CLI), "lint", "--dialect", "de", "--only", "GermanSpellCheck",
             "--format", "json", str(path)],
            capture_output=True, text=True,
        ).stdout
    if not out.strip():
        return set()
    return {lint["matched_text"] for lint in json.loads(out)[0]["lints"]}


def harper_pos(words: list[str]) -> dict[str, str]:
    """The part-of-speech letters `harper-cli metadata --brief` prints per word."""
    out = subprocess.run(
        [str(CLI), "metadata", "--dialect", "de", "--brief"],
        input="\n".join(words), capture_output=True, text=True,
    ).stdout
    pos = {}
    for line in out.splitlines():
        word, _, rest = line.partition(":")
        pos[word.strip()] = rest.split()[0] if rest.split() else ""
    return pos


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--dic", type=pathlib.Path, default=pathlib.Path("/usr/share/hunspell/de_DE.dic"))
    parser.add_argument("--aff", type=pathlib.Path, default=pathlib.Path("/usr/share/hunspell/de_DE.aff"))
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()

    hunspell = Hunspell(args.aff, args.dic)
    known = harper_words()
    lower = sorted(w for w in known if re.fullmatch(r"[a-zäöüß]{3,}", w))

    candidates = set()
    for word in lower:
        for prefix in PREFIXES:
            form = prefix + word
            # *ververbrennen*: a prefix on a word that already has one.
            if any(word.startswith(p) for p in PREFIXES if p == prefix):
                continue
            if form in known or form.capitalize() in known:
                continue
            if hunspell.accepts(form):
                candidates.add(form)

    rejected = rejected_by_harper(sorted(candidates)) & candidates
    # Accepted, but only as a noun: *versprach* passes as a compound of
    # *Vers* and *prach*, and then wants a capital letter.
    noun_only = {
        word for word, pos in harper_pos(sorted(candidates - rejected)).items()
        if "N" in pos and "V" not in pos and "J" not in pos
    }
    missing = sorted(rejected | noun_only)
    existing = DICT.read_text(encoding="utf-8")
    present = {line.partition("/")[0] for line in existing.splitlines()}
    lines = [
        f"{form}/~~{'J' if ADJECTIVE.search(form) else 'V'} {COMMENT}"
        for form in missing
        if form not in present
    ]
    print(f"{len(candidates)} hunspell-attested forms, {len(lines)} new entries")
    for line in lines[:40]:
        print("  " + line)
    if args.apply and lines:
        with DICT.open("a", encoding="utf-8") as f:
            f.write("\n".join(lines) + "\n")
        print(f"appended to {DICT}")


if __name__ == "__main__":
    main()
