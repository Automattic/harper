#!/usr/bin/env python3
"""Give the strong verbs' e/i forms their verb reading.

Verbs like *geben*, *nehmen*, *lesen*, *helfen* change the stem vowel from *e*
to *i* or *ie* in the second and third person singular and in the imperative:
*du gibst*, *er gibt*, *gib!*; *er nimmt*, *nimm!*; *er liest*, *lies!*. The
conjugation affixes cannot build these, so the dictionary holds them as words
of their own — and `strip_german_noun_readings.py` took the noun reading the
bulk import had given them and left nothing behind: `gib/~~h`, `nimm/~~h`,
`nimmt/~~hG`, `stiehlt/~~hG`. Without a part of speech no rule can see that
*nimm* is a verb, and `GermanStrongImperative` needs exactly that to tell
*Gebe mir das!* from a correct sentence.

The forms are found from the infinitives, and hunspell decides:

    infinitive   stem   i-stem   3rd person        imperative
    geben        geb    gib      gibt  st:gibt      gib  st:gibt fl:W
    nehmen       nehm   nimm     nimmt st:nimmt     nimm st:nimmt fl:W
    lesen        les    lies     liest st:liest     lies st:lies
    leben        leb    lieb     liebt st:lieben    -- regular, rejected

The third person has to be an entry of its own in hunspell (its stem is
itself), which is what separates *gibt* from *liebt*. The imperative has to be
analysed either as a plain entry or as the third person with the imperative
flag `W`. The second person is added when hunspell accepts it.

Each form gets the verb property `V`: appended to an existing entry, or
written as a new one.

    harper-core/src/language/german/scripts/add_german_strong_imperatives.py \\
        --dic index.dic --aff index.aff --apply

The `.dic`/`.aff` are the UTF-8 igerman98 files wooorm/dictionaries ships.
`--apply` writes; the default is a dry run. Idempotent.
"""

import argparse
import ctypes
import ctypes.util
import pathlib
import re

DICT = pathlib.Path(__file__).resolve().parent.parent / "dictionary.dict"
COMMENT = "# strong e/i form, hunspell-attested (add_german_strong_imperatives.py)"
LINE = re.compile(r"^(?P<word>[^/\s]+)/(?P<tilde>\*?~*)(?P<flags>\S*)(?P<rest>.*)$")


class Hunspell:
    def __init__(self, aff: pathlib.Path, dic: pathlib.Path):
        lib = ctypes.CDLL(ctypes.util.find_library("hunspell-1.7") or "libhunspell-1.7.so.0")
        lib.Hunspell_create.restype = ctypes.c_void_p
        lib.Hunspell_create.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        lib.Hunspell_spell.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        lib.Hunspell_analyze.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(ctypes.POINTER(ctypes.c_char_p)),
            ctypes.c_char_p,
        ]
        lib.Hunspell_free_list.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(ctypes.POINTER(ctypes.c_char_p)),
            ctypes.c_int,
        ]
        self.lib = lib
        self.handle = lib.Hunspell_create(str(aff).encode(), str(dic).encode())

    def accepts(self, word: str) -> bool:
        return self.lib.Hunspell_spell(self.handle, word.encode("utf-8")) != 0

    def analyses(self, word: str) -> list[str]:
        out = ctypes.POINTER(ctypes.c_char_p)()
        n = self.lib.Hunspell_analyze(self.handle, ctypes.byref(out), word.encode("utf-8"))
        result = [out[i].decode("utf-8").strip() for i in range(n)]
        self.lib.Hunspell_free_list(self.handle, ctypes.byref(out), n)
        return result


def i_stems(stem: str) -> list[str]:
    """*geb* → *gib*, *les* → *lies*, *nehm* → *nimm*: the last *e* raised."""
    at = stem.rfind("e")
    # A stem in *-t* gives no imperative of its own shape (*tritt*, *gilt*),
    # and is mostly a participle read as an infinitive (*gestellten*).
    if at < 0 or stem.endswith("t") or any(v in stem[at + 1:] for v in "aeiouäöü"):
        return []
    head, tail = stem[:at], stem[at + 1:]
    stems = [f"{head}i{tail}", f"{head}ie{tail}"]
    # *nehm* → *nimm*: the length mark goes and the consonant doubles.
    if tail.startswith("h") and len(tail) == 2:
        stems.append(f"{head}i{tail[1] * 2}")
    return stems


def strong_forms(hunspell: Hunspell, infinitive: str, nouns: set[str] = frozenset()) -> list[str]:
    """The imperative, third and second person of `infinitive`'s e/i stem.

    `nouns` are the capitalized headwords: a third person that is one of them
    in capitals is the noun, *besen* → *Biest*.
    """
    stem = infinitive[:-2]
    # The first person keeps the *e*: *ich gebe*. Without it the "infinitive"
    # is no verb — *besen* would give *bis* and *bist*.
    if not hunspell.accepts(f"{stem}e"):
        return []
    forms = []
    for i_stem in i_stems(stem):
        third = f"{i_stem}t"
        if third.capitalize() in nouns or f"st:{third}" not in hunspell.analyses(third):
            continue
        imperative = [
            a for a in hunspell.analyses(i_stem)
            if a == f"st:{i_stem}" or a.startswith(f"st:{third} fl:W")
        ]
        if not imperative:
            continue
        second = f"{i_stem}t" if i_stem[-1] in "sßzx" else f"{i_stem}st"
        forms += [i_stem, third] + ([second] if hunspell.accepts(second) else [])
    return forms


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--dic", type=pathlib.Path, required=True)
    parser.add_argument("--aff", type=pathlib.Path, required=True)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    hunspell = Hunspell(args.aff, args.dic)

    # The infinitives hunspell knows: its .dic lists each verb once.
    infinitives = set()
    entries = args.dic.read_text(encoding="utf-8").splitlines()[1:]
    nouns = {line.split("/")[0] for line in entries if line[:1].isupper()}
    for line in entries:
        word = line.split("/")[0]
        if re.fullmatch(r"[a-zäöüß]{2,}en", word) and "e" in word[:-2]:
            infinitives.add(word)
        elif re.fullmatch(r"[a-zäöüß]{2,}", word):
            # Verb stems carry the infinitive through an affix: *geb/…*.
            if hunspell.accepts(f"{word}en"):
                infinitives.add(f"{word}en")

    forms = sorted({f for inf in infinitives for f in strong_forms(hunspell, inf, nouns)})

    lines = DICT.read_text(encoding="utf-8").splitlines(keepends=True)
    index = {}
    for at, line in enumerate(lines):
        match = LINE.match(line.rstrip("\n"))
        if match:
            index.setdefault(match["word"], at)
    changed, added = 0, []
    for form in forms:
        at = index.get(form)
        if at is None:
            added.append(f"{form}/~~V {COMMENT}\n")
            continue
        match = LINE.match(lines[at].rstrip("\n"))
        if "V" in match["flags"]:
            continue
        lines[at] = f"{form}/{match['tilde']}{match['flags']}V{match['rest']}\n"
        changed += 1
    print(f"{len(forms)} strong e/i forms, {changed} entries gain V, {len(added)} new")
    for line in added[:30]:
        print("  +", line.rstrip())
    if args.apply:
        DICT.write_text("".join(lines + added), encoding="utf-8")


if __name__ == "__main__":
    main()
