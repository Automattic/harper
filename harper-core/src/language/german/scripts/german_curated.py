#!/usr/bin/env python3
"""Hand-written German word tables, written for this repository.

None of it comes from another dictionary: the genders of everyday nouns, the
nouns with several genders, the abbreviations and the names are lists a German
speaker writes down from knowledge, and they are what `build_german_dictionary.py`
lays over the Wikidata lexemes. Where a table disagrees with Wikidata, the table
wins.
"""

# Nouns that have exactly one gender in standard German; the table wins over Wikidata.
COMMON_NOUN_GENDERS = {}
_F = """
Absprache Adresse Ampel Analyse Anapher Anfahrt Angst Ansprache Antwort Anzahl Arbeit Art Aster Aufgabe
Auslage Ausnahme Ausstellung Auster Bahn Banane Bank Bedeutung Bedrängnis Befugnis Beilage Besorgnis Betrübnis
Bevölkerung Bewandtnis Biene Birne Bitte Bitternis Blase Blume Breite Bremse Brille Brust Brücke Buchse Butter
Bühne Dame Datei Decke Dose Ecke Ehe Ente Erdbeere Erde Erlaubnis Ersparnis Etage Fahrkarte Fahrt Familie
Farbe Faxnummer Finsternis Firma Flasche Fliege Folge Form Forschung Frage Frau Freiheit Freude Freundin Frist
Frucht Fäulnis Gabel Gans Geburt Geschichte Gesellschaft Gesundheit Grenze Gruppe Größe Gurke Haltestelle Hand
Hausaufgabe Haut Henne Hilfe Hochschule Hochzeit Hoffnung Hose Hälfte Höhe Idee Information Insel Jacke
Kartoffel Katze Kenntnis Kerze Kette Kirche Kirsche Klasse Krankheit Kreide Kreuzung Kuh Kultur Kunst Kälte
Küche Küste Lampe Lehrerin Liebe Linie Lippe Luft Länge Lösung Manier Mannschaft Mappe Maus Medizin Meinung
Menge Methode Miete Milch Minute Mitte Musik Mutter Mühe Münze Mütze Nachricht Nacht Nase Natur Niederlage
Note Nudel Nuss Oma Orange Order Partei Pause Person Pfanne Politik Prüfung Rechnung Regel Regierung Region
Reihe Reise Rolle Rose Ruhe Sache Schlange Schule Schulter Schwester Schüssel Seite Sekunde Sonne Sorge Soße
Sprache Stadt Stelle Stimme Stirn Straße Studentin Studie Stunde Summe Suppe Tafel Tante Tasche Tasse Tastatur
Theorie Tiefe Tochter Toilette Tomate Traube Trauer Treppe Tür Tüte Uhr Universität Unkenntnis Ursache Version
Viper Wahrheit Wand Wange Welt Wiese Wildnis Wirkung Wirrnis Wirtschaft Wissenschaft Woche Wohnung Wolke Wurst
Wurzel Wärme Zahl Zeile Zeit Zeitschrift Zeitung Zitrone Zunge Zwiebel Ärztin
""".split()
COMMON_NOUN_GENDERS.update(dict.fromkeys(_F, "F"))
_M = """
Abend Abschwung Albtraum Anfang Anzug Apfel Arm Artikel Arzt Ast Aufsatz Ausweis Bach Bahnhof Bahnsteig Balkon
Bauch Bauer Baum Becher Begriff Berg Bericht Beruf Betrieb Bildschirm Bleistift Blitz Boden Brief Bruder
Buchstabe Bus Bäcker Bär Chef Computer Daumen Dienstag Doktor Donner Donnerstag Drucker Durst Einfluss Enkel
Erfolg Essig Fahrer Fall Fehler Feind Film Finger Fisch Flug Flughafen Flur Fluss Freitag Freund Frieden
Frühling Fuchs Fuß Füller Garten Gast Gedanke Grad Großvater Grund Gürtel Hals Hase Herbst Herd Herr Himmel
Hof Hund Hunger Inhalt Junge Kaffee Keller Koch Koffer Kollege Konsum Konsument Kopf Kredit Krieg Kuchen
Kugelschreiber Käse Körper Kühlschrank Lehrer Lärm Löffel Magen Mann Mantel Markt Mensch Mittwoch Monat Mond
Montag Morgen Mund Nachbar Name Navigator Nebel Onkel Opa Ordner Ort Park Pass Pfeffer Plan Platz Polizist
Preis Professor Psychologe Pullover Punkt Radiergummi Ranzen Raum Rechner Regen Regenschirm Rekord Rock
Rucksack Rücken Saft Salat Samstag Sand Satz Schal Schein Schlaf Schlüssel Schnee Schrank Schuh Schwung
Schüler Sessel Sieg Sinn Sohn Sommer Sonntag Sourcecode Spaß Spiegel Sprung Staat Stein Stern Stock Strand
Streit Strumpf Student Stuhl Sturm Tag Tee Teller Teppich Termin Text Tisch Tod Ton Topf Tornister Traum
Umschwung Unterricht Urlaub Vater Verein Vertrag Vogel Vorhang Wagen Wald Weg Wein Wert Wind Winter Wolf
Wunsch Zahn Zaun Zucker Zug Zweck
""".split()
COMMON_NOUN_GENDERS.update(dict.fromkeys(_M, "M"))
_Z = """
Abendessen Auge Auto Baby Bad Banner Bein Beispiel Bett Bier Bild Blatt Blut Boot Brot Buch Center Dach Datum
Desaster Ding Dorf Drittel Ei Eis Eisen Ende Ergebnis Essen Fach Fahrrad Feld Fenster Fernsehen Feuer Fleisch
Flugzeug Foto Foyer Frühstück Gatter Gebiet Gebäude Gefühl Gehirn Geld Gemüse Gepäck Geschirr Geschäft Gesetz
Gesicht Getränk Gewicht Gewitter Gitter Glas Glück Gold Gras Haar Handtuch Handy Haus Heft Hemd Herz Holz
Hotel Huhn Inland Insekt Jahr Kaliber Kaninchen Kilo Kind Kino Kissen Klassenzimmer Kleid Knie Konto
Krankenhaus Kupfer Land Leben Leder Licht Lied Mal Manöver Medikament Meer Meerschweinchen Messer Metall
Mitglied Mittagessen Mittel Monster Motorrad Museum Mädchen Möbel Netz Obst Ohr Orchester Papier Pech Pferd
Plastik Plädoyer Portemonnaie Problem Programm Projekt Prozent Quartier Radio Rathaus Recht Regal Restaurant
Salz Schaf Schiff Schloss Schwein Semester Silber Sofa Spiel Spielzeug Studium Stück System Team Telefon
Theater Thema Ticket Tier Tuch Turnier Ufer Ungeheuer Unternehmen Unwetter Viertel Viertelfinale Volk Wasser
Wetter Wissen Wochenende Wort Wörterbuch Zeugnis Ziel Zimmer Öl
""".split()
COMMON_NOUN_GENDERS.update(dict.fromkeys(_Z, "Z"))

# Nouns that have several genders (der/das Teil): lemma -> gender letters.
MULTI_GENDER_NOUNS = {
    'Teil': 'MZ',
    'see': 'MF',
    'band': 'MFZ',
    'leiter': 'MF',
    'kiefer': 'MF',
    'tor': 'MZ',
    'erbe': 'MZ',
    'verdienst': 'MZ',
    'schild': 'MZ',
    'moment': 'MZ',
    'joghurt': 'MFZ',
    'mangel': 'MF',
    'hut': 'MF',
    'kunde': 'MF',
    'heide': 'MF',
    'stift': 'MZ',
    'mark': 'FZ',
    'reis': 'MZ',
    'tau': 'MZ',
    'virus': 'MZ',
    'meter': 'MZ',
    'liter': 'MZ',
    'keks': 'MZ',
    'silvester': 'MZ',
    'wehr': 'FZ',
    'bund': 'MZ',
    'golf': 'MZ',
    'modul': 'MZ',
    'angehörige': 'MF',
    'post': 'MF',
    'patch': 'MZ',
    'glauben': 'MZ',
    'single': 'MFZ',
    'pin': 'MF',
    'update': 'MZ',
    'kuddelmuddel': 'MZ',
    'wirrwarr': 'MZ',
}

# Abbreviations written without a stop: (letters, flags, expansion).
ABBREVIATIONS = [
    ('ggf', '~~2r', 'gegebenenfalls'),
    ('evtl', '~~2r', 'eventuell'),
    ('bspw', '~~2r', 'beispielsweise'),
    ('insb', '~~2r', 'insbesondere'),
    ('inkl', '~~2r', 'inklusive'),
    ('exkl', '~~2r', 'exklusive'),
    ('zzgl', '~~2r', 'zuzüglich'),
    ('urspr', '~~2r', 'ursprünglich'),
    ('eigtl', '~~2r', 'eigentlich'),
    ('ehem', '~~2r', 'ehemalig'),
    ('dt', '~~2J', 'deutsch'),
    ('engl', '~~2J', 'englisch'),
    ('frz', '~~2J', 'französisch'),
    ('ital', '~~2J', 'italienisch'),
    ('lat', '~~2J', 'lateinisch'),
    ('griech', '~~2J', 'griechisch'),
    ('russ', '~~2J', 'russisch'),
    ('tschech', '~~2J', 'tschechisch'),
    ('ndl', '~~2J', 'niederländisch'),
    ('schwed', '~~2J', 'schwedisch'),
    ('dän', '~~2J', 'dänisch'),
    ('norw', '~~2J', 'norwegisch'),
    ('türk', '~~2J', 'türkisch'),
    ('arab', '~~2J', 'arabisch'),
    ('hebr', '~~2J', 'hebräisch'),
    ('chin', '~~2J', 'chinesisch'),
    ('jap', '~~2J', 'japanisch'),
    ('kroat', '~~2J', 'kroatisch'),
    ('slowen', '~~2J', 'slowenisch'),
    ('rumän', '~~2J', 'rumänisch'),
    ('hg', '~~2', 'Herausgeber'),
    ('hrsg', '~~2', 'Herausgeber'),
    ('op', '~~2', 'Opus'),
    ('bd', '~~2', 'Band'),
    ('jh', '~~2', 'Jahrhundert'),
    ('jr', '~~2', 'Junior'),
    ('var', '~~2', 'varietas'),
    ('subsp', '~~2', 'subspecies'),
    ('ssp', '~~2', 'subspecies'),
]

# Abbreviations that are words only with their stop: (letters with stop, expansion).
PERIOD_ABBREVIATIONS = [
    ('Dr.', 'Doktor'),
    ('Prof.', 'Professor'),
    ('Hr.', 'Herr'),
    ('Fr.', 'Frau, Freitag'),
    ('Fa.', 'Firma'),
    ('Str.', 'Straße'),
    ('Nr.', 'Nummer'),
    ('Tel.', 'Telefon'),
    ('Mob.', 'Mobil'),
    ('Hbf.', 'Hauptbahnhof'),
    ('Bhf.', 'Bahnhof'),
    ('Pl.', 'Platz'),
    ('Abs.', 'Absender, Absatz'),
    ('Abt.', 'Abteilung'),
    ('Zi.', 'Zimmer'),
    ('Whg.', 'Wohnung'),
    ('St.', 'Sankt, Stück'),
    ('geb.', 'geboren'),
    ('gest.', 'gestorben'),
    ('verh.', 'verheiratet'),
    ('led.', 'ledig'),
    ('verw.', 'verwitwet'),
    ('gesch.', 'geschieden'),
    ('sen.', 'senior'),
    ('jun.', 'junior'),
    ('gegr.', 'gegründet'),
    ('ehem.', 'ehemalig'),
    ('Jan.', 'Januar'),
    ('Feb.', 'Februar'),
    ('Febr.', 'Februar'),
    ('Apr.', 'April'),
    ('Aug.', 'August'),
    ('Sept.', 'September'),
    ('Sep.', 'September'),
    ('Okt.', 'Oktober'),
    ('Nov.', 'November'),
    ('Dez.', 'Dezember'),
    ('Mo.', 'Montag'),
    ('Di.', 'Dienstag'),
    ('Mi.', 'Mittwoch'),
    ('Do.', 'Donnerstag'),
    ('Sa.', 'Samstag'),
    ('So.', 'Sonntag'),
    ('Std.', 'Stunde'),
    ('Sek.', 'Sekunde'),
    ('Tsd.', 'Tausend'),
    ('Mio.', 'Million'),
    ('Mrd.', 'Milliarde'),
    ('Pkt.', 'Punkt'),
    ('Ztr.', 'Zentner'),
    ('Pfd.', 'Pfund'),
    ('Stk.', 'Stück'),
    ('Jhdt.', 'Jahrhundert'),
    ('rd.', 'rund'),
    ('Abb.', 'Abbildung'),
    ('Anm.', 'Anmerkung'),
    ('Aufl.', 'Auflage'),
    ('Ausg.', 'Ausgabe'),
    ('Kap.', 'Kapitel'),
    ('Tab.', 'Tabelle'),
    ('Verf.', 'Verfasser'),
    ('Inh.', 'Inhaber'),
    ('Mitgl.', 'Mitglied'),
    ('pers.', 'persönlich'),
    ('Ziff.', 'Ziffer'),
    ('Abschn.', 'Abschnitt'),
    ('Sp.', 'Spalte'),
    ('Bsp.', 'Beispiel'),
    ('bspw.', 'beispielsweise'),
    ('bzgl.', 'bezüglich'),
    ('bzw.', 'beziehungsweise'),
    ('bes.', 'besonders'),
    ('betr.', 'betreffend'),
    ('bez.', 'bezüglich, bezahlt'),
    ('Anz.', 'Anzahl'),
    ('allg.', 'allgemein'),
    ('Abk.', 'Abkürzung'),
    ('abzgl.', 'abzüglich'),
    ('ggü.', 'gegenüber'),
    ('ugs.', 'umgangssprachlich'),
    ('gem.', 'gemäß'),
    ('lt.', 'laut'),
    ('usf.', 'und so fort'),
    ('mind.', 'mindestens'),
    ('urspr.', 'ursprünglich'),
    ('eigtl.', 'eigentlich'),
    ('wg.', 'wegen'),
    ('Gr.', 'Größe, griechisch'),
    ('Kl.', 'Klasse, klein'),
    ('dt.', 'deutsch'),
    ('frz.', 'französisch'),
    ('ital.', 'italienisch'),
    ('span.', 'spanisch'),
    ('russ.', 'russisch'),
    ('österr.', 'österreichisch'),
    ('schweiz.', 'schweizerisch'),
    ('europ.', 'europäisch'),
]

GIVEN_NAMES = """
Agnieszka Ahmed Ahmet Alexander Ali Alina Amelie Amira Ana Andrea Andreas Anja Anke Anna Annika Anton Antonia
Aylin Ayse Ayşe Ben Benedikt Benjamin Bernd Bettina Birgit Björn Bob Can Carina Carl Celina Charles Charlotte
Christian Christina Clara Claudia Dana Daniel Daniela David Deniz Dennis Detlef Dieter Dirk Dmitri Dominik
Dora Elena Elias Elif Elke Ella Emil Emilia Emily Emine Emir Emma Emre Enrico Eric Erik Erika Erna Esra Eva
Ewa Fabian Fatma Felix Finja Finn Fiona Florian Frank Franziska Frieda Friedrich Fritz Fynn Georg George
Gerhard Gertrud Giovanni Gisela Greta Günter Günther Hamza Hanna Hannah Hannes Hans Harry Hasan Hassan Heike
Heinrich Heinz Helena Helga Helmut Hendrik Henri Henrik Henry Hermann Holger Horst Hussein Ibrahim Ida Ilse
Inge Ingrid Isabel Isabell Ivan Jack Jacob Jakob James Jan Jana Janis Jannik Jannis Jasmin Jason Jennifer Jens
Jessica Joel Johann Johanna Johannes John Jonah Jonas Jonathan Josefine Josephine Jule Julia Julian Julius
Justin Jürgen Kai Karim Karin Karl Katarzyna Kate Katharina Kerstin Kevin Kilian Klara Klaus Konstantin
Kristina Lara Lars Lasse Laura Layla Lea Leah Lena Leni Lennard Lennart Lenny Leo Leon Leonard Leonie Levi
Leyla Lia Liam Lilli Lilly Lily Lina Linus Lio Lisa Liv Lotta Louis Louisa Luca Lucas Ludwig Luis Luisa Luise
Luka Lukas Lutz Magdalena Maja Malia Malte Manfred Marcel Marco Marcus Marek Maria Marie Mario Markus Marlene
Marlon Marta Martin Martina Mary Mathilda Mathis Matilda Mats Matteo Matthias Mattis Max Maximilian Mehmet
Melanie Merle Merve Mia Michael Michelle Mika Mike Mila Milan Milena Milo Mira Mohamed Mohammed Monika Moritz
Muhammad Murat Mustafa Nele Nick Nicklas Nico Nicole Niels Niklas Niko Nils Nina Noah Noel Nora Norbert Olaf
Ole Olga Olivia Omar Oscar Oskar Otto Pascal Patrick Paul Paula Pavel Peter Petra Philipp Pia Pietro Rafael
Rainer Ralf Ralph Raphael Renate Robert Robin Rolf Romy Rosa Ruth Sabine Samir Samuel Sandra Sandro Sara Sarah
Sascha Sebastian Selina Silas Silke Simon Simone Sofia Sofie Sophia Sophie Stefan Stefanie Stephanie Steve
Susan Susanne Sven Svenja Tanja Teresa Thea Theo Theresa Thomas Thorsten Tilda Till Tim Timo Tina Tobias Tom
Tomasz Tony Torsten Ursula Ute Uwe Valentin Valentina Vanessa Vera Victoria Viktoria Vincent Volker Werner
Wilhelm William Wolfgang Yannick Yasmin Yusuf Zeynep Zoe Zofia Ömer
""".split()
PLACES = """
Argentinien Brasilien Dubai Florida Ibiza Kalifornien Kanada Kenia Korea Kreta Mallorca Malta Marokko Mexiko
Neuseeland Rhodos Rügen Sylt Teneriffa Texas Thailand Tokio Tokyo Tunesien Usedom Vietnam Zypern Ägypten
""".split()
BRANDS = """
Adidas Aldi Amazon Android Apple Covid Discord Duden Ebay Edeka Facebook Google Ikea Instagram Lego Lidl
Microsoft Netflix Nike Nintendo PayPal PlayStation Playstation Puma Rewe Samsung Snapchat Spotify Teams TikTok
Twitch WhatsApp Wikipedia Windows Xbox YouTube Zalando Zoom eBay iPad iPhone
""".split()

PLURALIA_TANTUM = """
Alpen Annalen Einkünfte Eltern Ferien Finanzen Flitterwochen Gebrüder Gliedmaßen Großeltern Kinkerlitzchen
Kosten Leute Machenschaften Masern Memoiren Personalien Pflegeeltern Pocken Röteln Schwiegereltern Spesen
Stiefeltern Textilien Trümmer Unkosten Utensilien Wirren
""".split()

# Units and other symbols German prose writes in lower case or in their own case, never as nouns.
UNITS = """
cm mm dm km µm nm pm kg mg µg ml cl dl hl ms µs ns ha qm cbm
m² m³ cm² cm³ km² mm² Hz kHz MHz GHz THz W mW kW MW GW TW kWh MWh GWh TWh Wh V mV kV A mA Ω kΩ MΩ J kJ MJ GJ
N kN Pa hPa kPa MPa GPa bar mbar K °C °F kcal cal dB bit Byte kB MB GB TB PB Kbit Mbit Gbit kbit mol cd lm lx Bq Sv Gy
mio mrd rpm ppm EUR USD CHF GBP mmHg psi PS
""".split()

# Latin and Greek words that stand in German prose in lower case (de facto, Homo sapiens).
FOREIGN_TERMS = """
facto sapiens scientia circa alias versus vitae natura magna laude summa cum honoris causa status quo modus
operandi vivendi vice versa ergo ipso jure iure priori posteriori hoc hominem vitro vivo situ extenso memoriam
rasa tabula libris sine qua bona fide forma curriculum ceterum censeo mutatis mutandis nolens volens festum
anno domini mortem optimum maximum minimum ultimo primo alma mater terminus loco corpus delicti sui generis
genus species homo erectus habilis rex pax lux veritas carpe diem memento mori ante quem post factum
hominum cogito sum errare humanum est repetitio studiorum nomen omen verbatim exempli gratia
""".split()

# More abbreviations with their stop, for the pieces German strings together (z. B., d. h., u. a.).
MORE_PERIOD_ABBREVIATIONS = """
z. d. u. a. o. ä. v. m. vgl. usw. etc. ca. evtl. ggf. sog. ebd. ders. dies. ff. f. s. Ggf. Evtl. Vgl. Usw.
""".split()

# Strong and irregular verbs: infinitive, preterite, past participle and, where the vowel changes,
# the third person singular present. Used for verbs whose paradigm Wikidata does not list.
STRONG_VERBS = """
backen backte gebacken bäckt
befehlen befahl befohlen befiehlt
beginnen begann begonnen
beißen biss gebissen
bergen barg geborgen birgt
betrügen betrog betrogen
biegen bog gebogen
bieten bot geboten
binden band gebunden
bitten bat gebeten
blasen blies geblasen bläst
bleiben blieb geblieben
braten briet gebraten brät
brechen brach gebrochen bricht
brennen brannte gebrannt
bringen brachte gebracht
denken dachte gedacht
dringen drang gedrungen
dürfen durfte gedurft darf
empfangen empfing empfangen empfängt
empfehlen empfahl empfohlen empfiehlt
essen aß gegessen isst
fahren fuhr gefahren fährt
fallen fiel gefallen fällt
fangen fing gefangen fängt
fechten focht gefochten ficht
finden fand gefunden
flechten flocht geflochten flicht
fliegen flog geflogen
fliehen floh geflohen
fließen floss geflossen
fressen fraß gefressen frisst
frieren fror gefroren
gebären gebar geboren gebiert
geben gab gegeben gibt
gedeihen gedieh gediehen
gehen ging gegangen
gelingen gelang gelungen
gelten galt gegolten gilt
genesen genas genesen
genießen genoss genossen
geschehen geschah geschehen geschieht
gewinnen gewann gewonnen
gießen goss gegossen
gleichen glich geglichen
gleiten glitt geglitten
graben grub gegraben gräbt
greifen griff gegriffen
haben hatte gehabt hat
halten hielt gehalten hält
hängen hing gehangen
heben hob gehoben
heißen hieß geheißen
helfen half geholfen hilft
kennen kannte gekannt
klingen klang geklungen
kneifen kniff gekniffen
kommen kam gekommen
können konnte gekonnt kann
kriechen kroch gekrochen
laden lud geladen lädt
lassen ließ gelassen lässt
laufen lief gelaufen läuft
leiden litt gelitten
leihen lieh geliehen
lesen las gelesen liest
liegen lag gelegen
lügen log gelogen
meiden mied gemieden
messen maß gemessen misst
misslingen misslang misslungen
mögen mochte gemocht mag
müssen musste gemusst muss
nehmen nahm genommen nimmt
nennen nannte genannt
pfeifen pfiff gepfiffen
preisen pries gepriesen
raten riet geraten rät
reiben rieb gerieben
reißen riss gerissen
reiten ritt geritten
rennen rannte gerannt
riechen roch gerochen
ringen rang gerungen
rinnen rann geronnen
rufen rief gerufen
saufen soff gesoffen säuft
saugen sog gesogen
schaffen schuf geschaffen
scheiden schied geschieden
scheinen schien geschienen
schelten schalt gescholten schilt
schieben schob geschoben
schießen schoss geschossen
schlafen schlief geschlafen schläft
schlagen schlug geschlagen schlägt
schleichen schlich geschlichen
schleifen schliff geschliffen
schließen schloss geschlossen
schlingen schlang geschlungen
schmeißen schmiss geschmissen
schmelzen schmolz geschmolzen schmilzt
schneiden schnitt geschnitten
schreiben schrieb geschrieben
schreien schrie geschrien
schreiten schritt geschritten
schweigen schwieg geschwiegen
schwellen schwoll geschwollen schwillt
schwimmen schwamm geschwommen
schwinden schwand geschwunden
schwingen schwang geschwungen
schwören schwor geschworen
sehen sah gesehen sieht
senden sandte gesandt
singen sang gesungen
sinken sank gesunken
sinnen sann gesonnen
sitzen saß gesessen
sollen sollte gesollt soll
spinnen spann gesponnen
sprechen sprach gesprochen spricht
sprießen spross gesprossen
springen sprang gesprungen
stechen stach gestochen sticht
stehen stand gestanden
stehlen stahl gestohlen stiehlt
steigen stieg gestiegen
sterben starb gestorben stirbt
stinken stank gestunken
stoßen stieß gestoßen stößt
streichen strich gestrichen
streiten stritt gestritten
tragen trug getragen trägt
treffen traf getroffen trifft
treiben trieb getrieben
treten trat getreten tritt
trinken trank getrunken
tun tat getan tut
verderben verdarb verdorben verdirbt
verdrießen verdross verdrossen
vergessen vergaß vergessen vergisst
verlieren verlor verloren
verzeihen verzieh verziehen
wachsen wuchs gewachsen wächst
waschen wusch gewaschen wäscht
weichen wich gewichen
weisen wies gewiesen
wenden wandte gewandt
werben warb geworben wirbt
werden wurde geworden wird
werfen warf geworfen wirft
wiegen wog gewogen
winden wand gewunden
wissen wusste gewusst weiß
wollen wollte gewollt will
ziehen zog gezogen
zwingen zwang gezwungen
""".strip().splitlines()

# Wikidata lexemes that are words of the language but so rare that their forms only get in the way of
# common ones (the plural of *Gart* is spelled like *Garten*).
RARE_LEXEMES = {("Gart", "noun"), ("Cer", "noun"), ("karten", "verb"), ("fehl", "adj"), ("lang", "prep"), ("samen", "verb"), ("samt", "adv")}

# Words that are a function word and a noun in their own right and must keep both
# (*der Morgen*, *das Recht* stay nouns although *morgen* and *recht* are adverbs).
NOUN_AND_FUNCTION_WORD = """
Morgen Abend Mittag Nacht Weg Recht Mal Nachmittag Vormittag Teil Anfang Ende Gegenteil Abseits All Bar Bass Bitte
Dank Dicke Ehe Eins Ferne Feste Gegenüber Gen Gerade Heim Hoch Kraft Kreuz Laut Los Minus Nichts Ober Plus Sein
Selbst Samt Trotz Zeit Zuhause Hintern Gemach Pomade Spat Zack Bien Item
""".split()

# Latin and Greek plurals Wikidata lists under no lexeme of their own.
LATIN_PLURALS = """
Korpora Charakteristika Pronomina Termini Visa Themata Schemata Dogmata Stigmata Lexika Praktika Prinzipien
Medien Studien Zentren Museen Kommata Aromata Stadien Individuen Materialien Ministerien Kriterien Indizes
Indices Matrizen Zyklen Genera Species Opera Tempora Modi Casus Foci Radii Nuclei Alumni Rezepte
""".split()

# Common adjectives Wikidata has no lexeme for.
EXTRA_ADJECTIVES = """
modern komplex rein grob schlimm ober inner äußer hinter vorder mittler
""".split()

# Words whose lower-case spelling is the verb and never the noun that shares the letters (ist / Ist).
LOWERCASE_ONLY = ["ist", "dürfen", "können", "sollen", "wollen", "müssen", "mögen"]

# Noun lexemes whose letters are a verb form first (*Schrieb* is colloquial; *schrieb* is the preterite).
NOT_NOUNS = {"Schrieb"}

# Well-known people by the name they are known under: artists, composers, writers, thinkers, scientists.
PERSONS = """
Rembrandt Dürer Holbein Cranach Rubens Vermeer Monet Manet Renoir Cézanne Picasso Dalí Klee Kandinsky Beckmann Macke
Munch Raffael Michelangelo Leonardo Tizian Caravaggio Botticelli Giotto Goya Velázquez Gauguin Matisse Chagall Miró
Bach Beethoven Brahms Mozart Haydn Schubert Schumann Wagner Händel Bruckner Mahler Telemann Mendelssohn Chopin Liszt
Verdi Puccini Rossini Vivaldi Debussy Ravel Tschaikowsky Dvořák Smetana Sibelius Grieg Bizet Offenbach Gluck Weber
Goethe Schiller Lessing Kleist Heine Fontane Hesse Kafka Brecht Böll Grass Rilke Hölderlin Novalis Tieck Eichendorff
Storm Keller Grillparzer Nestroy Büchner Hauptmann Zweig Musil Joyce Shakespeare Dante Cervantes Tolstoi Dostojewski
Tschechow Orwell Montaigne Voltaire Rousseau Molière Hugo Balzac Flaubert Zola Proust Homer Vergil Ovid Cicero
Kant Hegel Marx Engels Nietzsche Schopenhauer Husserl Heidegger Platon Aristoteles Sokrates Descartes Spinoza Leibniz
Hume Locke Hobbes Freud Jung Einstein Newton Galilei Kepler Kopernikus Darwin Humboldt Röntgen Planck Heisenberg Bohr
Curie Pasteur Koch Gauß Euler Riemann Hilbert Turing Edison Tesla Faraday Maxwell Hawking Luther Calvin Zwingli
Cäsar Augustus Nero Karl Otto Friedrich Heinrich Maximilian Barbarossa Napoleon Bismarck Hitler Stalin Lenin Churchill
Edward Henry Richard George William Charles Elisabeth Viktoria Marx Merkel Adenauer Brandt Schmidt Kohl Schröder
""".split()

NOUN_AND_FUNCTION_WORD_SET = set(NOUN_AND_FUNCTION_WORD)

# Rivers, mountains and countries by their gender (*die Weser*, *der Rhein*, *die Türkei*).
GEO_GENDERS = {}
for _g, _words in (
    ("F", "Weser Donau Elbe Spree Isar Havel Saale Mosel Lahn Ems Leine Ruhr Aller Werra Fulda Eder Lippe "
          "Themse Seine Loire Rhone Garonne Wolga Newa Weichsel Memel Donau Zugspitze Eifel Rhön Pfalz Sahara "
          "Türkei Schweiz Ukraine Slowakei Mongolei Antarktis Arktis Toskana Lombardei Normandie Bretagne Provence"),
    ("M", "Rhein Main Neckar Inn Lech Kocher Harz Schwarzwald Bodensee Brocken Everest Vesuv Ätna Nil Amazonas "
          "Balkan Himalaya Kaukasus Jura Odenwald Spessart Taunus Westerwald Hunsrück Libanon Irak Iran Sudan Jemen"),
):
    for _w in _words.split():
        GEO_GENDERS[_w] = _g

# Common German family names (Wikidata lists the family names of notable people only).
FAMILY_NAMES = """
Müller Schmidt Schneider Fischer Weber Meyer Wagner Becker Schulz Hoffmann Schäfer Koch Bauer Richter Klein Wolf
Schröder Neumann Schwarz Zimmermann Braun Krüger Hofmann Hartmann Lange Schmitt Werner Schmitz Krause Meier
Lehmann Schmid Schulze Maier Köhler Herrmann König Walter Mayer Huber Kaiser Fuchs Peters Lang Scholz Möller
Weiß Jung Hahn Schubert Vogel Friedrich Keller Günther Frank Berger Winkler Roth Beck Lorenz Baumann Franke
Albrecht Schuster Simon Ludwig Böhm Winter Kraus Martin Schumacher Krämer Vogt Stein Jäger Otto Sommer Groß
Seidel Heinrich Brandt Haas Schreiber Graf Schulte Dietrich Ziegler Kuhn Kühn Pohl Engel Horn Busch Bergmann
Thomas Voigt Sauer Arnold Wolff Pfeiffer Meyer Lindner Maurer Hermann Ernst Peter Lutz Naumann Mayer Hein
Jansen Schütz Steiner Lenz Kramer Fischer Huber Wimmer Gruber Bachmann Mueller
""".split()

# Regular verbs Wikidata has no lexeme for.
EXTRA_VERBS = ["eichen"]

# Forms Wikidata does not list under the verb they belong to.
EXTRA_VERB_FORMS = ["worden"]

# Countries, regions and cities (Wikidata lists most under labels of several words: *Frankfurt am Main*).
MORE_PLACES = """
Deutschland Österreich Schweiz Frankreich Italien Spanien Portugal England Schottland Wales Irland Polen Tschechien
Slowakei Ungarn Rumänien Bulgarien Griechenland Türkei Russland Ukraine Weißrussland Litauen Lettland Estland
Finnland Schweden Norwegen Dänemark Island Niederlande Belgien Luxemburg Liechtenstein Monaco Malta Zypern Kroatien
Slowenien Serbien Bosnien Albanien Mazedonien Montenegro Kosovo Moldau Georgien Armenien Aserbaidschan Kasachstan
China Japan Korea Indien Pakistan Iran Irak Israel Palästina Syrien Libanon Jordanien Ägypten Libyen Tunesien
Algerien Marokko Nigeria Kenia Äthiopien Südafrika Brasilien Argentinien Chile Peru Kolumbien Venezuela Mexiko Kuba
Kanada Amerika Afrika Asien Europa Australien Neuseeland Bayern Baden Württemberg Hessen Sachsen Thüringen
Brandenburg Mecklenburg Vorpommern Niedersachsen Westfalen Rheinland Nordrhein Schleswig Holstein Saarland Franken
Schwaben Berlin Hamburg München Köln Frankfurt Stuttgart Düsseldorf Dortmund Essen Leipzig Bremen Dresden Hannover
Nürnberg Duisburg Bochum Wuppertal Bielefeld Bonn Münster Karlsruhe Mannheim Augsburg Wiesbaden Gelsenkirchen
Braunschweig Chemnitz Kiel Aachen Halle Magdeburg Freiburg Krefeld Lübeck Oberhausen Erfurt Mainz Rostock Kassel
Hagen Hamm Saarbrücken Mülheim Potsdam Ludwigshafen Oldenburg Leverkusen Osnabrück Solingen Heidelberg Herne Neuss
Darmstadt Paderborn Regensburg Ingolstadt Würzburg Fürth Wolfsburg Offenbach Ulm Heilbronn Pforzheim Göttingen
Bottrop Trier Recklinghausen Reutlingen Bremerhaven Koblenz Remscheid Jena Erlangen Siegen Hildesheim Salzgitter
Zürich Bern Basel Genf Lausanne Luzern Wien Graz Linz Salzburg Innsbruck Klagenfurt Paris Lyon Marseille London
Manchester Liverpool Rom Mailand Neapel Turin Florenz Venedig Madrid Barcelona Lissabon Porto Brüssel Amsterdam
Rotterdam Prag Warschau Krakau Budapest Bukarest Sofia Athen Istanbul Moskau Petersburg Kiew Stockholm Oslo
Kopenhagen Helsinki Dublin Edinburgh Peking Schanghai Tokio Seoul Delhi Bombay Kairo Washington Chicago Boston
Houston Dallas Atlanta Miami Seattle Kalifornien Texas Florida Virginia Carolina Georgia Ohio Nevada Arizona Oregon
Montana Alaska Hawaii Michigan Illinois Indiana Louisiana Kentucky Tennessee Mississippi Alabama Maryland
Pennsylvania Connecticut Vermont Delaware Colorado Utah Wyoming Idaho Kansas Nebraska Oklahoma Missouri Iowa Minnesota
Wisconsin Mecklenburg Schleswig Rheinland Nordrhein Pfalz Hunsrück Eifel Harz Allgäu Tirol Vorarlberg Kärnten
Steiermark Burgenland Bukowina Siebenbürgen Böhmen Mähren Schlesien Pommern Ostpreußen Westpreußen Sudeten
""".split()

# Verbs with a weak and a strong paradigm (*erschreckt* transitive, *erschrickt* intransitive): the weak one is
# the one Harper checks against.
WEAK_AND_STRONG = {"erschrecken"}
