#!/usr/bin/env python3
"""Give the strong verbs' e/i and umlaut forms their verb reading.

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

Each form gets the verb property `V`, and the forms on the changed stem the
marker `%` as well, which the rules read: appended to an existing entry, or
written as a new one. The second person plural on the plain stem (*ihr
lauft*) is added as a plain verb where the affixes missed it. A noun reading on a form whose capitalized spelling
hunspell does not know (*flichtst/~~NA*) is the bulk import's and goes.

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
COMMENT = "# strong verb form, hunspell-attested (add_german_strong_imperatives.py)"
NOUN_ONLY_PROPERTIES = set("NMFZA+")
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
    """*geb* → *gib*, *les* → *lies*, *nehm* → *nimm*, *tret* → *tritt*: the
    last *e* raised. The same spelling as `raised_stems` in `grammar/verbs.rs`."""
    at = stem.rfind("e")
    if at < 0 or any(v in stem[at + 1:] for v in "aeiouäöü"):
        return []
    head, tail = stem[:at], stem[at + 1:]
    stems = [f"{head}i{tail}", f"{head}ie{tail}"]
    # *nehm* → *nimm*: the length mark goes and the consonant doubles.
    if tail.startswith("h") and len(tail) == 2:
        stems.append(f"{head}i{tail[1] * 2}")
    # *tret* → *tritt*: the short vowel doubles the *t*.
    if tail == "t":
        stems.append(f"{head}itt")
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
    # A stem in *-t* must show the verb's own *-et* (*ihr tretet*): adjectives
    # in *-ten* (*besten*, *schlechten*) and participles (*gestellten*) do not.
    if stem.endswith("t") and not hunspell.accepts(f"{stem}et"):
        return []
    forms = []
    for i_stem in i_stems(stem):
        # A stem in *-t* is its own third person: *tritt*, *gilt*.
        third = i_stem if i_stem.endswith("t") else f"{i_stem}t"
        # *Tritt* is a noun as well; a stem in *-t* has passed the *-et* test.
        is_noun = third.capitalize() in nouns and not stem.endswith("t")
        # The raised stem must have no infinitive of its own: *richt* is
        # *richten*, *ritt* is *reiten*.
        if is_noun or hunspell.accepts(f"{i_stem}en") or not any(
            a == f"st:{third}" or a.startswith(f"st:{third} fl:")
            # *gilt*, *ficht*: hunspell files them under the second person.
            or (i_stem == third and a.startswith(f"st:{third}st "))
            for a in hunspell.analyses(third)
        ):
            continue
        imperative = [
            a for a in hunspell.analyses(i_stem)
            if a == f"st:{i_stem}" or a.startswith(f"st:{third} fl:W")
            or (i_stem == third and a.startswith((f"st:{third} fl:", f"st:{third}st ")))
        ]
        if not imperative:
            continue
        second = f"{i_stem}t" if i_stem[-1] in "sßzx" else f"{third}st"
        forms += [i_stem, third] + ([second] if hunspell.accepts(second) else [])
    return forms


def umlauted_forms(hunspell: Hunspell, infinitive: str, nouns: set[str] = frozenset()) -> list[str]:
    """The third and second person of `infinitive`'s umlauted stem: *fahren*
    → *fährt*, *fährst*; *halten* → *hält*, *hältst*; *stoßen* → *stößt*.

    Only verbs with no weak preterite — *backte*, *fragte* exist, so *backt*
    and *fragt* stand beside *bäckt* and *frägt* — and the umlauted stem must
    have no infinitive of its own (*zählt* is *zählen*'s). The imperative
    keeps the plain vowel, so there is none to add.
    """
    stem = infinitive[:-2]
    if (
        not hunspell.accepts(f"{stem}e")
        or hunspell.accepts(f"{stem}te")
        or (stem[-1:] in "td" and hunspell.accepts(f"{stem}ete"))
        or (stem[-1:] in "td" and not hunspell.accepts(f"{stem}et"))
    ):
        return []
    at = max(stem.rfind("a"), stem.rfind("o"))
    if at < 0 or any(c in "aeiouäöü" and c != "u" for c in stem[at + 1:]):
        return []
    umlauted = stem[:at] + ("ä" if stem[at] == "a" else "ö") + stem[at + 1:]
    third = umlauted if umlauted.endswith("t") else f"{umlauted}t"
    second = f"{umlauted}t" if umlauted[-1] in "sßzx" else f"{umlauted}st"
    if (
        third.capitalize() in nouns
        # *fällen* is an infinitive of its own; *trägen* is only *träge*
        # declined.
        or f"st:{umlauted}en" in hunspell.analyses(f"{umlauted}en")
        or not hunspell.accepts(third)
        or not hunspell.accepts(second)
        or not any(a.startswith(f"st:{third}") for a in hunspell.analyses(third))
    ):
        return []
    return [third, second]


def plural_on_plain_stem(hunspell: Hunspell, stem: str) -> list[str]:
    """The second person plural keeps the plain stem: *ihr lauft*, *ihr gebt*,
    *ihr haltet*. The conjugation affixes miss some of them (*lauft*)."""
    plural = f"{stem}et" if stem[-1:] in "td" else f"{stem}t"
    return [plural] if hunspell.accepts(plural) else []


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

    strong = set()
    plural = set()
    for inf in infinitives:
        changed = strong_forms(hunspell, inf, nouns) + umlauted_forms(hunspell, inf, nouns)
        if changed:
            strong |= set(changed)
            plural |= set(plural_on_plain_stem(hunspell, inf[:-2]))
    plural -= strong
    forms = sorted(strong | plural)

    lines = DICT.read_text(encoding="utf-8").splitlines(keepends=True)
    index = {}
    for at, line in enumerate(lines):
        match = LINE.match(line.rstrip("\n"))
        if match:
            index.setdefault(match["word"], at)
    changed, added = 0, []
    for form in forms:
        at = index.get(form)
        # The changed-stem forms are marked `%`; the plain plural only needs
        # the verb reading.
        wanted = "V%" if form in strong else "V"
        if at is None:
            added.append(f"{form}/~~{wanted} {COMMENT}\n")
            continue
        match = LINE.match(lines[at].rstrip("\n"))
        flags = match["flags"]
        # *fahrt* is also the noun *Fahrt* written small; a verb reading would
        # hide *nahm fahrt auf* from the capitalization rule, and *ihr fahrt*
        # is recognized by its *ihr* anyway.
        if form not in strong and "N" in flags:
            continue
        # A noun reading on a form hunspell has no noun for is the bulk
        # import's: *flichtst/~~NA*. *Eintritt* is a noun, *eintritt* keeps it.
        if form.capitalize() not in nouns:
            flags = "".join(c for c in flags if c not in NOUN_ONLY_PROPERTIES)
        flags += "".join(c for c in wanted if c not in flags)
        if flags == match["flags"]:
            continue
        lines[at] = f"{form}/{match['tilde']}{flags}{match['rest']}\n"
        changed += 1
    print(f"{len(forms)} strong verb forms, {changed} entries gain V, {len(added)} new")
    for line in added[:30]:
        print("  +", line.rstrip())
    if args.apply:
        DICT.write_text("".join(lines + added), encoding="utf-8")


if __name__ == "__main__":
    main()
