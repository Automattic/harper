#!/usr/bin/env python3
"""Add the abbreviations German prose writes with a trailing full stop.

`ggf.`, `engl.`, `hg.`, `op.`, `Bd.`, `Nr.` and their kind are ordinary German
and were reported as misspellings: the tokenizer hands the linter the letters
without the stop, and `dictionary.dict` has no entry for them. igerman98 is no
help here -- it does not list them either -- so this is a curated table, with the
expansion in the comment of every entry it writes.

Flag `2` is the abbreviation property (`abbreviation: true` in the metadata).
`GermanNounCapitalization` rejects anything carrying it outright, which is the
point: `hg` must never be "corrected" to `Hg`.

Words that are *also* an ordinary German noun are deliberately absent. `Alb`,
`Pol`, `Port`, `Ungar` and `Finn` are all real nouns, and giving the lower-case
spelling the abbreviation flag would silence a correct capitalization lint. The
few citations that use them stay flagged; that is the cheaper mistake.

So is anything that is also the **start of German words**. Any dictionary word
of three characters or more may act as a compound element, so `versch`
(verschieden) made `verschwand` decompose into `versch` + `wand` -- no longer a
misspelling, a compound noun with a capitalization lint on it. `abb`, `geb`,
`gest`, `anm`, `verh`, `aufl`, `syn` and `kap` are the same shape and between
them let a hundred generated typos through `just language-recall german`; they
are out for that reason, not because they are rare. Check a candidate before
adding it:

    a="geb"
    sum(1 for w in words if w.startswith(a) and w[len(a):] in words)   # prefix
    sum(1 for w in words if w.endswith(a) and w[:-len(a)] in words)    # suffix

Check **both** ends. `sen` (Senior) reads as a prefix in only 22 words and as a
suffix in 16641 -- it is the ending of every `-sen` plural and infinitive German
has -- and on its own it accounted for most of the typo leakage.

Excluding abbreviations from compounding outright was tried and is worse: the
German dictionary leans on loose compounding for proper-name coverage, and
`Leitha`, `Omaha` and `Himmerland` are only words because `ha` is one.

Run from the repo root. `--apply` writes; the default is a dry run. Re-running is
idempotent.
"""

import pathlib
import sys

DICT = pathlib.Path("harper-core/src/language/german/dictionary.dict")

# The abbreviation property, plus a part of speech where the expansion has an
# unambiguous one: `J` adjective, `r` adverb. Reference abbreviations stand for
# nouns but are written lower case in citations, so they carry the property
# alone -- a noun reading would put them straight back in front of the
# capitalization rule.
ABBREVIATIONS = [
    # adverbs
    ("ggf", "~~2r", "gegebenenfalls"),
    ("evtl", "~~2r", "eventuell"),
    ("bspw", "~~2r", "beispielsweise"),
    ("insb", "~~2r", "insbesondere"),
    ("inkl", "~~2r", "inklusive"),
    ("exkl", "~~2r", "exklusive"),
    ("zzgl", "~~2r", "zuzüglich"),
    ("urspr", "~~2r", "ursprünglich"),
    ("eigtl", "~~2r", "eigentlich"),
    ("ehem", "~~2r", "ehemalig"),
    # language adjectives, as used in glosses: "(engl. pressure)"
    ("dt", "~~2J", "deutsch"),
    ("engl", "~~2J", "englisch"),
    ("frz", "~~2J", "französisch"),
    ("ital", "~~2J", "italienisch"),
    ("lat", "~~2J", "lateinisch"),
    ("griech", "~~2J", "griechisch"),
    ("russ", "~~2J", "russisch"),
    ("tschech", "~~2J", "tschechisch"),
    ("ndl", "~~2J", "niederländisch"),
    ("schwed", "~~2J", "schwedisch"),
    ("dän", "~~2J", "dänisch"),
    ("norw", "~~2J", "norwegisch"),
    ("türk", "~~2J", "türkisch"),
    ("arab", "~~2J", "arabisch"),
    ("hebr", "~~2J", "hebräisch"),
    ("chin", "~~2J", "chinesisch"),
    ("jap", "~~2J", "japanisch"),
    ("kroat", "~~2J", "kroatisch"),
    ("slowen", "~~2J", "slowenisch"),
    ("rumän", "~~2J", "rumänisch"),
    # bibliographic and biographical references
    ("hg", "~~2", "Herausgeber"),
    ("hrsg", "~~2", "Herausgeber"),
    ("op", "~~2", "Opus"),
    ("bd", "~~2", "Band"),
    ("jh", "~~2", "Jahrhundert"),
    ("jr", "~~2", "Junior"),
    # taxonomy
    ("var", "~~2", "varietas"),
    ("subsp", "~~2", "subspecies"),
    ("ssp", "~~2", "subspecies"),
]


# Abbreviations that are only words with their full stop: `Nr. 5`, `Dr. Meier`,
# `29. Okt. 2011`. They are entered *with* the stop, and `GermanSpellCheck`
# accepts the letters only when the stop follows directly. That keeps the bare
# letters a misspelling (`Tel` is not a word, `Tel.` is) and keeps them out of
# compounds, which is what the table above has to guard against by hand: an
# entry with a stop in it can never be a compound element. So nouns (`Art.`,
# `Bd.`) and the starts and ends of words (`Abb.`, `Kap.`) are safe here. The
# abbreviation of a noun is capitalized, like the noun.
PERIOD_ABBREVIATIONS = [
    # addresses, titles, letters
    ("Dr.", "Doktor"), ("Prof.", "Professor"), ("Hr.", "Herr"), ("Fr.", "Frau, Freitag"),
    ("Fa.", "Firma"), ("Str.", "Straße"), ("Nr.", "Nummer"), ("Tel.", "Telefon"),
    ("Mob.", "Mobil"), ("Hbf.", "Hauptbahnhof"), ("Bhf.", "Bahnhof"), ("Pl.", "Platz"),
    ("Abs.", "Absender, Absatz"), ("Abt.", "Abteilung"), ("Zi.", "Zimmer"),
    ("Whg.", "Wohnung"), ("St.", "Sankt, Stück"), ("geb.", "geboren"), ("gest.", "gestorben"),
    ("verh.", "verheiratet"), ("led.", "ledig"), ("verw.", "verwitwet"), ("gesch.", "geschieden"),
    ("sen.", "senior"), ("jun.", "junior"), ("gegr.", "gegründet"), ("ehem.", "ehemalig"),
    # months and weekdays
    ("Jan.", "Januar"), ("Feb.", "Februar"), ("Febr.", "Februar"), ("Apr.", "April"),
    ("Aug.", "August"), ("Sept.", "September"), ("Sep.", "September"), ("Okt.", "Oktober"),
    ("Nov.", "November"), ("Dez.", "Dezember"), ("Mo.", "Montag"), ("Di.", "Dienstag"),
    ("Mi.", "Mittwoch"), ("Do.", "Donnerstag"), ("Sa.", "Samstag"), ("So.", "Sonntag"),
    # quantities and units
    ("Std.", "Stunde"), ("Sek.", "Sekunde"), ("Tsd.", "Tausend"), ("Mio.", "Million"),
    ("Mrd.", "Milliarde"), ("Pkt.", "Punkt"), ("Ztr.", "Zentner"), ("Pfd.", "Pfund"),
    ("Stk.", "Stück"), ("Jhdt.", "Jahrhundert"), ("rd.", "rund"),
    # references
    ("Abb.", "Abbildung"), ("Anm.", "Anmerkung"), ("Aufl.", "Auflage"), ("Ausg.", "Ausgabe"),
    ("Kap.", "Kapitel"), ("Tab.", "Tabelle"), ("Verf.", "Verfasser"), ("Inh.", "Inhaber"),
    ("Mitgl.", "Mitglied"), ("pers.", "persönlich"), ("Ziff.", "Ziffer"), ("Abschn.", "Abschnitt"),
    ("Sp.", "Spalte"), ("Bsp.", "Beispiel"), ("bspw.", "beispielsweise"), ("bzgl.", "bezüglich"),
    ("bzw.", "beziehungsweise"), ("bes.", "besonders"), ("betr.", "betreffend"),
    ("bez.", "bezüglich, bezahlt"), ("Anz.", "Anzahl"), ("allg.", "allgemein"),
    ("Abk.", "Abkürzung"), ("abzgl.", "abzüglich"), ("ggü.", "gegenüber"), ("ugs.", "umgangssprachlich"),
    ("gem.", "gemäß"), ("lt.", "laut"), ("usf.", "und so fort"), ("mind.", "mindestens"),
    ("urspr.", "ursprünglich"), ("eigtl.", "eigentlich"), ("wg.", "wegen"), ("Gr.", "Größe, griechisch"),
    ("Kl.", "Klasse, klein"), ("dt.", "deutsch"), ("frz.", "französisch"), ("ital.", "italienisch"),
    ("span.", "spanisch"), ("russ.", "russisch"), ("österr.", "österreichisch"),
    ("schweiz.", "schweizerisch"), ("europ.", "europäisch"),
]

# The flags the first import gave entries with a stop (`bes./~~NYE`): a noun
# reading and two plural affixes, none of which an abbreviation has.
STALE_PERIOD_FLAGS = "~~NYE"


def add_period_abbreviations(lines: list[str], apply: bool) -> list[str]:
    """Normalize the stale entries with a stop, then add the missing ones.

    An abbreviation of a noun is written with a capital (*Prof.*, *Std.*), and a
    capitalized entry is matched case-sensitively, so *Herr prof. Müller* stays
    a misspelling. A lower-case entry of the same abbreviation would let it
    through, so it is dropped in favour of the capitalized one.
    """
    capitalized = {w.lower() for w, _ in PERIOD_ABBREVIATIONS if w[0].isupper()}
    out = []
    fixed = 0
    for line in lines:
        word, sep, rest = line.partition("/")
        if sep and word.endswith(".") and word in capitalized:
            fixed += 1
            continue
        if sep and word.endswith(".") and rest.split(" ", 1)[0].strip() == STALE_PERIOD_FLAGS:
            line = f"{word}/~~2\n"
            fixed += 1
        out.append(line)
    known = {line.split("/", 1)[0].strip() for line in out if "/" in line}
    missing = [(w, e) for w, e in PERIOD_ABBREVIATIONS if w not in known]
    print(f"{fixed} stale entries with a stop normalized, {len(missing)} to add")
    out += [f"{w}/~~2 # abbreviation with its full stop: {e}\n" for w, e in missing]
    return out


def main() -> int:
    if not DICT.exists():
        print(f"{DICT} not found -- run from the repo root", file=sys.stderr)
        return 1

    lines = DICT.read_text(encoding="utf-8").splitlines(keepends=True)
    known = {line.split("/", 1)[0].strip() for line in lines if "/" in line}

    missing = [entry for entry in ABBREVIATIONS if entry[0] not in known]
    print(f"{len(ABBREVIATIONS) - len(missing)} already present, {len(missing)} to add")
    for word, flags, expansion in missing[:10]:
        print(f"    {word}/{flags}  # {expansion}")

    lines += [f"{word}/{flags} # abbreviation: {expansion}\n" for word, flags, expansion in missing]
    lines = add_period_abbreviations(lines, "--apply" in sys.argv)

    if "--apply" in sys.argv:
        DICT.write_text("".join(lines), encoding="utf-8")
        print(f"wrote {DICT}")
    else:
        print("dry run; pass --apply to write")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
