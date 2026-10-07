#!/usr/bin/env python3
"""Record the gender of common everyday nouns the corpus oracle never reached.

The oracle in `audit_german_gender.py` reads genders off articles in running
text and is exhausted on the available corpus. Everyday school and household
vocabulary is under-represented there, so `Tasche`, `Garten` or `Urlaub` carry
no gender and `GermanDeterminerGender` stays silent on them.

The table below is hand-written and holds only nouns with ONE gender in
standard German; a word with several (*der/das Teil*) belongs in
`add_german_multi_gender_nouns.py`.

Behaviour, per entry in the table:

* the dictionary has a noun entry without any gender -> the flag is added;
* it already has the same gender -> nothing happens;
* it has a DIFFERENT gender -> the table wins and the change is printed. The
  table only holds words whose gender is not in doubt, and the audit this
  produced found *Kuchen*, *Mund* and *Traum* recorded neuter, *Bier*, *Messer*
  and *Ufer* masculine, and thirty-odd words (*Bein*, *Bett*, *Finger*, *Löffel*,
  *Telefon*) recorded with two genders where German has one;
* it has no noun entry of its own -> reported as missing, or, for the few in
  `NEW_LEMMAS`, appended as a lemma.

Gender flags are plain properties (`M`, `F`, `Z`), not affixes, so adding one
builds no surface form. Idempotent.

    python3 harper-core/src/language/german/scripts/add_german_common_noun_genders.py
"""

import pathlib
import re
import sys

DICT = pathlib.Path(__file__).resolve().parent.parent / "dictionary.dict"

M, F, Z = "M", "F", "Z"

TABLE = {}


def add(gender, words):
    for word in words.split():
        TABLE[word] = gender


# fmt: off
add(M, """Hund Garten Urlaub Brief Tisch Stuhl Lehrer Schüler Freund Vater Bruder Onkel Sohn Opa Junge Mann Mensch
Tag Morgen Abend Montag Dienstag Mittwoch Donnerstag Freitag Samstag Sonntag Monat Sommer Winter Herbst Frühling
Mantel Pullover Schuh Strumpf Rock Anzug Gürtel Schal Rucksack Koffer Schlüssel Regenschirm
Apfel Kuchen Käse Salat Zucker Kaffee Tee Saft Fisch Teller Löffel Becher Topf Herd Kühlschrank
Baum Wald Berg Fluss Bach Strand Himmel Regen Schnee Wind Sturm Mond Stern Hase Vogel Wolf Bär Fuchs
Kopf Arm Finger Fuß Bauch Rücken Hals Zahn Mund Magen Körper
Weg Bahnhof Zug Bus Bahnsteig Flughafen Platz Markt Park Hof Stock Keller Balkon Flur Spiegel Boden Zaun
Computer Bildschirm Bleistift Kugelschreiber Radiergummi Füller Tornister Ranzen Satz Text Aufsatz Fehler Buchstabe Unterricht
Wagen Fahrer Arzt Polizist Bäcker Koch Bauer Nachbar Gast Chef Doktor Erfolg Spaß Streit Traum Schlaf Hunger Durst Lärm Preis Plan Name Wunsch Gedanke Krieg Frieden""")
add(F, """Katze Tasche Schule Frau Mutter Schwester Tante Tochter Oma Freundin Lehrerin Familie Klasse Stunde Pause Prüfung Aufgabe Hausaufgabe Note Frage Antwort Geschichte Sprache Zeitung Zeitschrift Tafel Kreide Mappe Tür Wand Decke Treppe Küche Wohnung Straße Stadt Brücke Kirche Wiese Blume Rose Sonne Erde Luft Welt Natur Insel Küste Maus Kuh Ente Gans Henne Biene Fliege Schlange
Jacke Hose Brille Mütze Uhr Kette Lampe Flasche Tasse Gabel Kerze Tüte Dose Schüssel Pfanne Suppe Wurst Butter Milch Banane Birne Kirsche Tomate Gurke Kartoffel Zwiebel Orange Zitrone
Nase Hand Schulter Brust Zunge Lippe Stirn Wange Gesundheit Krankheit Medizin
Woche Minute Sekunde Stunde Nacht Zeit Mitte Reise Fahrt Arbeit Mühe Hilfe Bitte Idee Meinung Liebe Freude Angst Sorge Ruhe Kälte Wärme Farbe Zahl Summe Seite Zeile Reihe Ecke Linie Größe Länge Breite Höhe Tiefe Kunst Musik Wahrheit Freiheit""")
add(Z, """Haus Buch Heft Blatt Papier Auto Fahrrad Motorrad Flugzeug Schiff Boot Kind Baby Mädchen Tier Pferd Schwein Schaf Huhn Kaninchen Meerschweinchen Insekt Zimmer Bad Fenster Dach Bett Sofa Regal Licht Feuer Wasser Eis Brot Ei Fleisch Obst Gemüse Bier Glas Messer Geschirr Frühstück Mittagessen Abendessen Essen Getränk Spiel Spielzeug Lied Bild Foto Telefon Handy Radio Fernsehen Wetter Jahr Wochenende Datum Ergebnis Beispiel Wort Wörterbuch Thema Fach Zeugnis Klassenzimmer Gesicht Auge Ohr Herz Bein Knie Gehirn Blut Leben Geld Geschäft Dorf Land Meer Ufer Gras Gebäude Museum Theater Kino Hotel Restaurant Krankenhaus Rathaus Schloss Ziel Stück Ding Problem Gefühl Wissen Kleid Hemd Tuch Handtuch Kissen Gepäck""")
# fmt: on

# Lemma -> flags. `N` noun, `h` compound-capable, `E` plural -n, plus the gender.
NEW_LEMMAS = {
    "katze": "NhEF",
    "kirche": "NhEF",
    "kirsche": "NhEF",
    "decke": "NhEF",
    "freude": "NhEF",
    "note": "NhEF",
    "tiefe": "NhEF",
    "auge": "NhEZ",
}

LINE = re.compile(r"^(?P<word>[^/\s]+)/(?P<head>~*)(?P<flags>\S*)(?P<rest>.*)$")
LETTER = {"M": "masculine", "F": "feminine", "Z": "neuter"}


def main() -> int:
    lines = DICT.read_text(encoding="utf-8").split("\n")
    added, same, conflicts = 0, 0, []
    seen = set()
    for i, line in enumerate(lines):
        m = LINE.match(line)
        if not m:
            continue
        key = next((w for w in (m["word"], m["word"].capitalize()) if w in TABLE), None)
        if key is None or m["word"].lower() != key.lower():
            continue
        flags = m["flags"]
        # A noun entry, or an entry that is used as one: the genitive and
        # plural affixes only make sense on a noun. Lines that merely
        # inflect a verb (`# REPLACES bergen`) are not.
        is_noun = "N" in flags or set("MFZ") & set(flags) or set("0XYE") & set(flags)
        if not is_noun or "REPLACES" in m["rest"]:
            continue
        seen.add(key)
        recorded = set("MFZ") & set(flags)
        wanted = TABLE[key]
        if not recorded:
            lines[i] = f"{m['word']}/{m['head']}{flags}{wanted}{m['rest']}"
            added += 1
        elif recorded == {wanted}:
            same += 1
        else:
            rest_flags = "".join(c for c in flags if c not in "MFZ")
            lines[i] = f"{m['word']}/{m['head']}{rest_flags}{wanted}{m['rest']}"
            conflicts.append((key, wanted, "".join(sorted(recorded))))

    # Nouns the dictionary only knows as the plural of a stem (`katz` + -e
    # reads as *Katze*, number PLURAL) get a lemma of their own. A flag on the
    # stem is wrong: `tief` is an adjective and `deck` a verb stem.
    present = {m["word"] for line in lines if (m := LINE.match(line))}
    appended = []
    for lemma, flags in NEW_LEMMAS.items():
        if lemma not in present:
            appended.append(f"{lemma}/~~{flags} # everyday noun, gender from add_german_common_noun_genders.py")
            seen.add(lemma.capitalize())
            added += 1
    if appended:
        while lines and lines[-1] == "":
            lines.pop()
        lines += appended + [""]
    DICT.write_text("\n".join(lines), encoding="utf-8")

    print(f"added {added}, already right {same}", file=sys.stderr)
    for word, wanted, got in conflicts:
        print(f"CORRECTED {word}: dictionary {got} -> {wanted}", file=sys.stderr)
    missing = sorted(set(TABLE) - seen)
    print(f"no noun entry ({len(missing)}): {' '.join(missing)}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
