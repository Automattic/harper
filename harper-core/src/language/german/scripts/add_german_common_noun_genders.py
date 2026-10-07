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

A second round added about 200 nouns of general prose; it found *Ende*
recorded feminine (every *das Ende* was a report), *Leder* and *Silber*
masculine, and thirteen feminine `-nis` nouns recorded neuter.

`UMLAUT_PLURAL_SINGULARS` gets the singular property `A`: the `-en`/`-er`/`-el`
nouns whose plural carries an umlaut, so that *die Garten* can be reported (see
`grammar/noun_gender.rs`).

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
# Frequent nouns of general prose, the second round of the audit. *Ende* was
# recorded feminine, which made every *das Ende* a report.
add(M, """Anfang Fall Grund Kopf Staat Krieg Zweck Ort Raum Begriff Satz Artikel Bericht Ordner Rechner Wert Markt
Betrieb Beruf Kollege Vertrag Kredit Verein Feind Nachbar Herr Großvater Enkel Tod Körper Daumen Wunsch Ton Film
Sieg Erfolg Punkt Rekord Termin Flug Pass Ausweis Schein Stock Schrank Sessel Teppich Vorhang Drucker
Professor Student Einfluss Sinn Inhalt Grad Stern Nebel Blitz Donner Stein Sand Ast Pfeffer Essig Wein""")
add(F, """Frage Art Stelle Seite Hand Geschichte Kultur Wirtschaft Politik Regierung Partei Gesellschaft Region Grenze
Lösung Idee Nachricht Information Datei Version Zahl Firma Rechnung Bank Sache Gruppe Person Bevölkerung Dame
Ehe Hochzeit Geburt Ärztin Haut Stimme Hoffnung Bühne Ausstellung Mannschaft Niederlage Uhr Frist Bahn Straße
Kreuzung Ampel Haltestelle Fahrkarte Münze Wohnung Miete Tastatur Universität Hochschule Studentin Wissenschaft
Forschung Studie Theorie Methode Analyse Regel Ausnahme Folge Ursache Wirkung Rolle Bedeutung Form Menge Anzahl
Hälfte Wolke Wurzel Frucht Traube Erdbeere Nuss Soße Nudel Toilette Etage""")
add(Z, """Ende Leben Mal Recht Gesetz Volk System Programm Projekt Ziel Mittel Gebiet Feld Wort Netz Unternehmen Konto
Ding Stück Team Mitglied Baby Krankenhaus Medikament Herz Blut Haar Knie Gesicht Gefühl Glück Pech Lied Bild Kino
Museum Spiel Ergebnis Datum Ticket Schloss Möbel Telefon Papier Heft Studium Beispiel Gewicht Drittel Viertel
Prozent Kilo Gewitter Eis Feuer Holz Metall Eisen Gold Silber Plastik Leder Obst Salz Öl Getränk""")

# Feminine nouns the dictionary records as masculine or neuter only. Each one
# turns a correct *eine Erlaubnis* into a report, and, read the other way, a
# correct *die Erlaubnis gilt* into a plural subject with a singular verb.
add(F, """Erlaubnis Kenntnis Besorgnis Betrübnis Bewandtnis Ersparnis Fäulnis Wirrnis Bitternis Bedrängnis Befugnis
Finsternis Wildnis Beilage Auslage Aster Auster Anapher""")
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
    # The rest of the table had no noun entry at all: compounds the base
    # dictionary only reaches by decomposition (*Bahnhof*, *Kühlschrank*), and
    # nouns that only survived as another word class (*Morgen* the adverb,
    # *Regen* the verb, *Mühe* dropped as "never capitalized"). `A` marks a
    # singular whose umlaut plural the affixes cannot build; `X`, `Y`, `E`
    # and `b` are the plural affixes and mark the base singular themselves.
    "abendessen": "NhZ",
    "bahnhof": "NhMA",
    "bahnsteig": "NhMX",
    "bildschirm": "NhMX",
    "bleistift": "NhMX",
    "fernsehen": "NhZ",
    "flughafen": "NhMA",
    "frühstück": "NhZX",
    "handtuch": "NhZA",
    "hausaufgabe": "NhFE",
    "herbst": "NhMX",
    "klassenzimmer": "NhZ",
    "krankenhaus": "NhZA",
    "kugelschreiber": "NhM",
    "kühlschrank": "NhMA",
    "morgen": "NhM",
    "motorrad": "NhZA",
    "mühe": "NhFE",
    "ranzen": "NhM",
    "regen": "NhM",
    "regenschirm": "NhMX",
    "schal": "NhMb",
    "spielzeug": "NhZX",
    "wochenende": "NhZE",
    "wörterbuch": "NhZA",
    "zeitschrift": "NhFY",
    "region": "NhFY",
    "nachricht": "NhFY",
    "hochzeit": "NhFY",
    "kollege": "NhME",
    "vorhang": "NhMA",
    "großvater": "NhMA",
    "hochschule": "NhFE",
    "fahrkarte": "NhFE",
    "erdbeere": "NhFE",
}

# Nouns in `-en`, `-er` and `-el` whose plural is spelled with an umlaut
# (*Garten*/*Gärten*, *Vogel*/*Vögel*). The spelling of most such nouns says
# nothing about their number, *der Lehrer*/*die Lehrer*, so
# `GermanDeterminerGender` leaves them alone; these get the singular property
# `A` so that *die Garten* can be read as the error it is. *Wagen*, *Kasten*
# and *Bogen* are left out: both plurals are in use.
UMLAUT_PLURAL_SINGULARS = """Garten Vogel Apfel Bruder Mantel Vater Boden Ofen Hafen Faden Schaden Hammer
Sattel Nagel Schnabel Acker Schwager Kloster Laden Graben Mutter Tochter""".split()

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

    singular_marked = 0
    for i, line in enumerate(lines):
        m = LINE.match(line)
        if not m or m["word"].capitalize() not in UMLAUT_PLURAL_SINGULARS:
            continue
        flags = m["flags"]
        is_noun = "N" in flags or set("MFZ") & set(flags)
        if is_noun and "REPLACES" not in m["rest"] and "A" not in flags:
            lines[i] = f"{m['word']}/{m['head']}{flags}A{m['rest']}"
            singular_marked += 1

    # Nouns the dictionary only knows as the plural of a stem (`katz` + -e
    # reads as *Katze*, number PLURAL) get a lemma of their own. A flag on the
    # stem is wrong: `tief` is an adjective and `deck` a verb stem.
    # A line for the same word that is not a noun (`morgen/~~r`, `mühe/~~h`)
    # does not count: the dictionary merges two lines of one word.
    appended = []
    for lemma, flags in NEW_LEMMAS.items():
        if lemma.capitalize() not in seen:
            appended.append(f"{lemma}/~~{flags} # everyday noun, gender from add_german_common_noun_genders.py")
            seen.add(lemma.capitalize())
            added += 1
    if appended:
        while lines and lines[-1] == "":
            lines.pop()
        lines += appended + [""]
    DICT.write_text("\n".join(lines), encoding="utf-8")

    print(f"added {added}, already right {same}, marked singular {singular_marked}", file=sys.stderr)
    for word, wanted, got in conflicts:
        print(f"CORRECTED {word}: dictionary {got} -> {wanted}", file=sys.stderr)
    missing = sorted(set(TABLE) - seen)
    print(f"no noun entry ({len(missing)}): {' '.join(missing)}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
