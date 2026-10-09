#!/usr/bin/env python3
"""Add the given names, places and brands German school writing is full of.

The spell checker reported *Emma*, *Mia*, *Ben*, *Finn*, *Leon*, *Max*, *Tom*,
*Lisa* and *Lena* — the most frequent children's names in Germany — as
misspellings, and with them *WhatsApp*, *Instagram* and *Netflix*. On the
correct example sentences of LanguageTool's German rules, proper names were the
largest single class of spelling false alarms. Corpus Wikipedia articles name
historical figures, which is why the import from igerman98 and the corpus never
reached these.

Each entry is written **capitalized**, so only the capitalized spelling is
accepted: a lower-case entry `ben` would hide the typo in *Ich ben müde*. The
flags are `N` (noun), `A` (singular) and `H` (genitive *-s*: *Emmas Buch*),
except for names that end in a sibilant, which take the apostrophe genitive
(*Max' Fahrrad*) and get no `H`.

A name is skipped when the dictionary already has an entry for it in either
case. Idempotent.

    python3 harper-core/src/language/german/scripts/add_german_given_names.py
"""

import pathlib
import re
import sys

DICT = pathlib.Path(__file__).resolve().parent.parent / "dictionary.dict"

# fmt: off
GIVEN_NAMES = """
Anna Lena Lea Leonie Lisa Laura Julia Sarah Sara Hannah Hanna Emma Mia Sophie Sophia Sofia Marie Maria Lara
Lina Clara Klara Emilia Johanna Katharina Charlotte Paula Jana Nina Melanie Sandra Sabine Petra Claudia Andrea
Susanne Monika Ursula Renate Karin Birgit Heike Anja Tanja Jessica Jennifer Vanessa Michelle Jasmin Nele Greta
Ida Frieda Mila Ella Luisa Louisa Amelie Antonia Elena Helena Isabel Isabell Jule Pia Svenja Franziska Christina
Kristina Stefanie Stephanie Nicole Daniela Martina Simone Elke Silke Ute Gisela Ingrid Helga Erika Gertrud Anke
Kerstin Bettina Carina Theresa Teresa Emily Lilly Lilli Lily Lotta Matilda Mathilda Marlene Magdalena Romy Zoe
Leni Luise Alina Annika Celina Fiona Finja Josephine Josefine Lia Liv Malia Maja Merle Milena Nora Rosa Selina
Thea Tilda Valentina Victoria Viktoria Elif Zeynep Ayşe Fatma Merve Esra Aylin Leyla Layla Amira Yasmin Sofie
Tom Max Paul Leon Lukas Lucas Luca Luka Jonas Finn Fynn Felix Ben Noah Elias Luis Louis Henry Henri Emil Anton
Theo Moritz Jakob Jacob Niklas Nicklas Nils Niels Tim Timo Jan Julian Philipp David Daniel Alexander Maximilian
Sebastian Tobias Florian Fabian Simon Kevin Dennis Marcel Patrick Stefan Michael Thomas Andreas Markus Marcus
Peter Klaus Hans Jürgen Wolfgang Uwe Frank Dieter Günter Günther Horst Helmut Karl Carl Heinz Werner Manfred
Rainer Bernd Holger Thorsten Torsten Matthias Christian Martin Johannes Benjamin Jonathan Leonard Oskar Oscar
Ole Mats Lars Erik Eric Hannes Till Malte Henrik Hendrik Kai Sven Björn Dirk Jens Olaf Ralf Ralph Rolf Lutz Detlef
Volker Norbert Gerhard Wilhelm Friedrich Heinrich Hermann Georg Ludwig Otto Fritz Mohammed Muhammad Mohamed
Ali Ahmed Ahmet Mustafa Mehmet Yusuf Emre Emir Can Deniz Ömer Hamza Omar Ibrahim Hassan Hussein Karim Samir
Milan Matteo Mattis Mathis Linus Levi Liam Leo Lio Jannik Yannick Janis Jannis Joel Johann Jonah Julius Kilian
Konstantin Lennard Lennart Lenny Lasse Marlon Mika Milo Nick Nico Niko Noel Ole Pascal Rafael Raphael Samuel
Silas Valentin Vincent Benedikt Dominik Jason Justin Robin Sascha Sandro Marco Mario Enrico Giovanni Pietro
John Mike Mary James Robert Tony Jack Harry George William Charles Emily Olivia Jessica Kate Susan Steve Bob
Eva Dana Olga Vera Tina Dora Erna Ilse Inge Ruth Marta Mira Leah Ana Ewa Zofia Agnieszka Katarzyna Ayse Emine
Ivan Dmitri Pavel Marek Tomasz Hasan Murat
"""

PLACES = """
Tokyo Tokio Mallorca Ibiza Sylt Rügen Usedom Teneriffa Kreta Rhodos Malta Zypern Dubai Florida Kalifornien
Texas Kanada Mexiko Brasilien Argentinien Ägypten Marokko Tunesien Kenia Thailand Vietnam Korea Neuseeland
"""

BRANDS = """
WhatsApp Instagram Netflix Spotify YouTube TikTok Snapchat Facebook Google Amazon Apple Microsoft Samsung
iPhone iPad Android Windows PlayStation Playstation Xbox Nintendo Lego Lidl Aldi Rewe Edeka Ikea Nike Adidas
Puma Zalando Ebay eBay PayPal Wikipedia Discord Twitch Zoom Teams Duden Covid
"""
# fmt: on

# Lower-case words a name would otherwise capture. *Max* the name made lower-case
# *max* (maximal) a capitalization report, *passen max 5 Leute* -> *Max*.
SHADOWED_ABBREVIATIONS = {"max": "2r # abbreviation: maximal"}

LINE = re.compile(r"^(?P<word>[^/\s#]+)/")


def main() -> int:
    text = DICT.read_text(encoding="utf-8")
    known = {m["word"].lower() for line in text.split("\n") if (m := LINE.match(line))}

    seen = set()
    new_lines = []
    for source, words in (("given name", GIVEN_NAMES), ("place", PLACES), ("brand", BRANDS)):
        for word in words.split():
            if word.lower() in known or word in seen:
                continue
            seen.add(word)
            flags = "NA" if word[-1] in "sxzß" else "NHA"
            new_lines.append(f"{word}/~~{flags} # {source}, add_german_given_names.py")

    present = {m["word"] for line in text.split("\n") if (m := LINE.match(line))}
    for word, rest in SHADOWED_ABBREVIATIONS.items():
        if word not in present:
            new_lines.append(f"{word}/~~{rest}")

    if new_lines:
        text = text.rstrip("\n") + "\n" + "\n".join(new_lines) + "\n"
        DICT.write_text(text, encoding="utf-8")
    print(f"added {len(new_lines)} names", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
