use crate::language::german::grammar::noun_phrase;
use crate::{
    Punctuation, Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::linting::german_foreign_stretch,
    language::german::spell::lexical_classes::{FOREIGN_TERMS, NUMERALS, UNIT_ABBREVIATIONS},
    language::morphology::{MorphologyExt, NumberSet},
    linting::{Lint, LintKind, Linter, Suggestion},
    spell::Dictionary,
};

/// A linter that checks to make sure German nouns are capitalized.
/// In German, all nouns must be capitalized (not just proper nouns like in English).
///
/// # Noun / verb (and noun / adjective) homographs
///
/// Many German words are a noun in one context and a verb or adjective in
/// another: *"Der **Fang** ist groß"* (noun) vs *"..., **fang** an"* (verb),
/// *"der **Halt**"* vs *"**halt** still"*. The dictionary cannot tell the two
/// apart on its own — worse, Harper's `WordId` lower-cases spellings, so the
/// lowercase verb reading is merged with the capitalized noun entry and almost
/// every candidate ends up carrying *both* a noun and a verb reading.
///
/// This linter therefore only asks the dictionary for a noun reading and then
/// decides using **syntactic context**:
///
/// * A word with a clean, unambiguous noun reading (noun, but no verb /
///   adjective / adverb reading) is flagged wherever it appears.
/// * A word that is *also* a verb / adjective / adverb is flagged only when the
///   token to its left licenses a noun phrase — an article, another determiner,
///   a possessive, a preposition or a spelled-out number.
pub struct GermanNounCapitalization<T>
where
    T: Dictionary,
{
    dictionary: T,
    /// Suffixes that strongly indicate a noun, paired with minimum word length
    /// to avoid false positives on short function words.
    noun_suffixes: Vec<(Vec<char>, usize)>,
}

// The spelled-out numerals, unit abbreviations and lower-case foreign terms
// that used to live here are now dictionary data, carried by the property flags
// `1`, `2` and `3` in `dictionary.dict` / `annotations.json`. They are read back
// as sets by `spell::lexical_classes`; adding a word no longer means editing Rust.

/// Separable / directional verb prefixes. A lowercase word that starts with one
/// of these and ends in a finite-verb shape (`herausrückt`, `hinausläuft`) is a
/// verb, not a noun — even if the compound-aware dictionary decomposes it.
const SEPARABLE_VERB_PREFIXES: &[&str] = &[
    "heraus",
    "herein",
    "hinaus",
    "hinein",
    "hervor",
    "hinauf",
    "hinab",
    "herauf",
    "herab",
    "herunter",
    "hinunter",
    "herüber",
    "hinüber",
    "zurück",
    "zusammen",
    "auseinander",
    "entgegen",
    "empor",
    "voran",
    "voraus",
    "vorbei",
    "vorüber",
];

/// Personal pronouns that stand alone as the subject of a clause. A German main
/// clause puts the finite verb second, so a lowercase word straight after one of
/// these is that verb — never a miscapitalized noun.
///
/// This matters because the first-person singular ending `-e` has exactly the
/// shape of a noun singular or plural: *die Rede*, *die Spiele*, *die Fahre*.
/// Without this gate the compound-aware dictionary hands back a noun reading for
/// *rede*, *spiele*, *trinke*, *zeige* and *fahre*, and `Ich fahre nach Berlin`
/// is reported as a capitalization error.
///
/// `ihr` is deliberately absent: it is also the possessive determiner, so *ihr
/// haus* really is a miscapitalized noun. The apposition the rest of the list
/// gives up on — *wir Deutschen*, *du Dummkopf* — needs a capital anyway, and is
/// far rarer than a first-person sentence.
const SUBJECT_PRONOUNS: &[&str] = &["ich", "du", "er", "sie", "es", "wir"];

/// Nouns that German writes lower case when they are the predicate of a
/// copula or of *tun*: *es tut mir **leid***, *mir ist **angst** und
/// **bange***, *er ist **schuld***, *die Firma ist **pleite***, *jemandem
/// **feind** sein*. Duden lists them as nouns in adjectival use, and the rule
/// is a closed class — which is why it is a list rather than a flag.
///
/// Each is paired with the verbs that license it. *Ich habe keine Angst* has
/// none of them and stays reportable, as does *seine angst ist groß*, where a
/// determiner shows the word is the noun.
const PREDICATIVE_NOUNS: &[(&str, &[&str])] = &[
    ("leid", TUN),
    ("angst", COPULAS),
    ("bange", COPULAS),
    ("schuld", COPULAS),
    ("pleite", COPULAS),
    ("feind", COPULAS),
    ("freund", COPULAS),
    ("gram", COPULAS),
];

/// A coordinator in front of a clause leaves its verb-second order intact:
/// *Und das **tat** verdammt gut*.
const CLAUSE_COORDINATORS: &[&str] = &["und", "oder", "aber", "denn", "sondern", "doch"];

const TUN: &[&str] = &[
    "tun", "tut", "tue", "tust", "tat", "taten", "tätest", "täte", "getan",
];

const COPULAS: &[&str] = &[
    "ist", "sind", "bin", "bist", "seid", "war", "waren", "warst", "wart", "sei", "wäre", "wären",
    "sein", "gewesen", "wird", "werden", "wirst", "werdet", "wurde", "wurden", "ward", "würde",
    "geworden", "bleibt", "bleiben", "blieb", "blieben",
];

const LANGUAGE_GLOSS_MARKERS: &[&str] = &[
    "deutsch",
    "althochdeutsch",
    "mittelhochdeutsch",
    "niederdeutsch",
    "hochdeutsch",
    "altdeutsch",
    "englisch",
    "altenglisch",
    "französisch",
    "altfranzösisch",
    "italienisch",
    "spanisch",
    "portugiesisch",
    "niederländisch",
    "lateinisch",
    "mittellateinisch",
    "spätlateinisch",
    "neulateinisch",
    "kirchenlateinisch",
    "griechisch",
    "altgriechisch",
    "neugriechisch",
    "dänisch",
    "schwedisch",
    "norwegisch",
    "isländisch",
    "russisch",
    "polnisch",
    "tschechisch",
    "ungarisch",
    "finnisch",
    "türkisch",
    "arabisch",
    "hebräisch",
    "persisch",
    "japanisch",
    "chinesisch",
    "koreanisch",
    "indogermanisch",
    "urgermanisch",
    "germanisch",
    "gotisch",
    "keltisch",
    "sanskrit",
];

/// Adjective / adverb / participle final segments that a lowercase common noun
/// essentially never ends with. Used to veto a noun reading even when the
/// compound-aware dictionary hands one back — decomposable adjective compounds
/// such as `massenproduzierbar` otherwise resolve to a bare noun.
fn has_non_noun_ending(s: &str) -> bool {
    let n = s.chars().count();
    // Adjective / adverb / participle suffixes.
    (s.ends_with("bar") && n >= 5)          // machbar, produzierbar, nachprüfbar
        || (s.ends_with("sam") && n >= 6)   // langsam, gemeinsam
        || (s.ends_with("haft") && n >= 6)  // dauerhaft, lebhaft
        || (s.ends_with("los") && n >= 6)   // arbeitslos, hilflos
        || (s.ends_with("lich") && n >= 6)  // eigentlich, wesentlich
        || s.ends_with("weise")             // schrittweise, teilweise, beziehungsweise
        || s.ends_with("wärts")             // rückwärts, vorwärts
        || (s.ends_with("isch") && n >= 7)  // technisch, kritisch (not "Tisch", "Fisch")
        || (s.ends_with("end") && n >= 8)   // schwimmend, abschließend (not "Abend", "Jugend")
        || (s.ends_with("ig") && n >= 10)   // modellabhängig, temperaturabhängig
        || (s.ends_with("nahe") && n >= 6)  // zeitnahe, praxisnahe
        || (s.ends_with("uelle") && n >= 6) // rituelle, aktuelle, individuelle
        || (s.ends_with("öse") && n >= 6)   // amouröse, nervöse, grandiose
        || (s.ends_with("frei") && n >= 7)  // latexfrei, barrierefrei, schadstofffrei
        // High-precision finite-verb / participle endings. Corpus mining added
        // many of these to the dictionary as bare "~~Nh" nouns.
        || (s.ends_with("iert") && n >= 6)  // funktioniert, existiert, studiert
        || (s.ends_with("ßt") && n >= 5)    // fließt, heißt, genießt, schießt
        || (s.ends_with("mmt") && n >= 5)   // kommt, bestimmt, stimmt
        || (s.ends_with("nnt") && n >= 5)   // kennt, brennt, erkennt, benennt
        || (s.ends_with("elt") && n >= 7 && !s.ends_with("welt")) // entwickelt, behandelt
        || (s.ends_with("ert") && n >= 8 && !s.ends_with("wert")) // erläutert, geändert, gefördert (not "Konzert")
}

impl<T: Dictionary> GermanNounCapitalization<T> {
    pub fn new(dictionary: T) -> Self {
        let noun_suffixes = vec![
            (vec!['h', 'e', 'i', 't'], 5),           // -heit (min 5 chars)
            (vec!['k', 'e', 'i', 't'], 5),           // -keit
            (vec!['u', 'n', 'g'], 5),                // -ung
            (vec!['n', 'i', 's'], 5),                // -nis
            (vec!['t', 'u', 'm'], 5),                // -tum
            (vec!['l', 'i', 'n', 'g'], 6),           // -ling
            (vec!['i', 'o', 'n'], 5),                // -ion
            (vec!['t', 'ä', 't'], 5),                // -tät
            (vec!['s', 'c', 'h', 'a', 'f', 't'], 8), // -schaft
        ];

        Self {
            dictionary,
            noun_suffixes,
        }
    }

    fn is_non_noun(word_lower: &[char]) -> bool {
        let s: String = word_lower.iter().collect();
        noun_phrase::is_lowercase_non_noun(&s)
    }

    /// Does `prev` end a clause subject, making this token the finite verb?
    ///
    /// See [`SUBJECT_PRONOUNS`]. The pronoun has to be the written word itself,
    /// so a capitalized *Sie* counts and so does a sentence-initial *Ich*.
    fn follows_subject_pronoun(prev: Option<&Token>, document: &Document) -> bool {
        prev.is_some_and(|p| {
            matches!(p.kind, TokenKind::Word(_))
                && SUBJECT_PRONOUNS.contains(&noun_phrase::lowercase_of(p, document).as_str())
        })
    }

    /// The token range of the clause around `index`, up to the punctuation on
    /// either side.
    fn clause_bounds(tokens: &[&Token], index: usize) -> (usize, usize) {
        let is_boundary = |token: &&Token| matches!(token.kind, TokenKind::Punctuation(_));
        let start = tokens[..index]
            .iter()
            .rposition(is_boundary)
            .map_or(0, |at| at + 1);
        let end = tokens[index + 1..]
            .iter()
            .position(is_boundary)
            .map_or(tokens.len(), |at| index + 1 + at);
        (start, end)
    }

    /// The words of the clause around `index`, up to the punctuation on
    /// either side, lower-cased, without the word at `index` itself.
    fn clause_around(tokens: &[&Token], index: usize, document: &Document) -> Vec<String> {
        let (start, end) = Self::clause_bounds(tokens, index);
        (start..end)
            .filter(|&at| at != index && matches!(tokens[at].kind, TokenKind::Word(_)))
            .map(|at| noun_phrase::lowercase_of(tokens[at], document))
            .collect()
    }

    /// Is this noun/verb homograph the finite verb of a verb-second clause?
    ///
    /// German puts the finite verb second, so in *Das **bedarf** noch der
    /// Klärung*, *Der **bestand** aus Stahl*, *Keiner **macht** hier
    /// Hausaufgaben* the word behind a clause-initial pronoun is the verb. The
    /// chunker reads *das bedarf* as a determiner and its head, and nothing on
    /// the two words says otherwise; the rest of the clause does. When no
    /// other word in it can be a verb, this one is.
    ///
    /// Two conditions keep it from swallowing real errors. The word in front
    /// has to be one that can be a **subject pronoun** — *Vielen dank* opens
    /// with a dative and stays reportable. And it has to **open the clause**:
    /// *aus der reihe*, *pro stunde* are not in that position, which matters
    /// because many finite forms (*kostet*, *tanzt*) carry no verb reading in
    /// the dictionary, so "no other verb" alone is weak evidence.
    fn is_verb_second(tokens: &[&Token], index: usize, document: &Document) -> bool {
        const PRONOUN_OPENERS: &[&str] = &[
            "der", "die", "das", "dies", "diese", "dieser", "dieses", "jener", "jene", "jenes",
            "keiner", "keine", "keines", "jeder", "jede", "jedes", "einer", "eine", "eines",
            "alle", "alles", "beide", "viele", "manche", "einige", "welche", "wer", "was",
        ];
        let (start, end) = Self::clause_bounds(tokens, index);
        let opener_at = if start < index
            && CLAUSE_COORDINATORS
                .contains(&noun_phrase::lowercase_of(tokens[start], document).as_str())
        {
            start + 1
        } else {
            start
        };
        if index != opener_at + 1
            || !PRONOUN_OPENERS
                .contains(&noun_phrase::lowercase_of(tokens[opener_at], document).as_str())
        {
            return false;
        }
        // The pronoun is third person, so its verb ends in *-t* (*macht*,
        // *bedarf* aside, *tat*) or is a strong preterite in *-d* (*bestand*).
        // *Die türme von Hanoi* and *Keine sorge* end in *-e*, a first person
        // at best. And a verb-second clause goes on behind its verb: *Das
        // gerät, mit dem ich arbeite* stops at the comma.
        let word = noun_phrase::lowercase_of(tokens[index], document);
        let third_person = word.ends_with('t') || word.ends_with('d') || word.ends_with("arf");
        let clause_goes_on = index + 1 < end;
        if !third_person || !clause_goes_on {
            return false;
        }
        !(start..end).filter(|&at| at != index).any(|at| {
            let token = tokens[at];
            token.kind.is_verb()
                && document
                    .get_span_content(&token.span)
                    .first()
                    .is_some_and(|c| c.is_lowercase())
        })
    }

    /// Is this one of [`PREDICATIVE_NOUNS`] in the position German writes it
    /// lower case: no determiner in front, a licensing verb in the clause?
    fn is_predicative_noun(tokens: &[&Token], index: usize, document: &Document) -> bool {
        let word = noun_phrase::lowercase_of(tokens[index], document);
        let Some((_, verbs)) = PREDICATIVE_NOUNS.iter().find(|(noun, _)| *noun == word) else {
            return false;
        };
        // *mein bester freund* is the noun: a determiner or an attributive
        // adjective in front shows it.
        let after_attribute = index.checked_sub(1).is_some_and(|at| {
            noun_phrase::supplies_determiner(tokens[at], document) || tokens[at].kind.is_adjective()
        });
        !after_attribute
            && Self::clause_around(tokens, index, document)
                .iter()
                .any(|other| verbs.contains(&other.as_str()))
    }

    /// Is this token inside a stretch of a foreign language?
    ///
    /// See [`german_foreign_stretch`] for what counts as evidence and why.
    fn in_foreign_stretch(tokens: &[&Token], index: usize, document: &Document) -> bool {
        german_foreign_stretch::in_foreign_stretch(tokens, index, document, |token| {
            noun_phrase::supplies_determiner(token, document)
        })
    }

    /// Is the token glued to a hyphen on either side?
    ///
    /// German suspends the shared part of coordinated compounds and marks the
    /// gap with a hyphen: *"auf **welt-**, **volks-**, **stadt-** und
    /// hauswirtschaftlicher Ebene"*, *"Konfliktverhütung und **-lösung**"*.
    /// Each fragment is a compound element, correctly lower case, and the
    /// tokenizer hands them over as bare words. Requires the hyphen to be
    /// directly adjacent so that a dash used as punctuation — set off by spaces
    /// — does not suppress a real noun.
    ///
    /// A bracketed infix does the same: *"Holz(über)schuh"*,
    /// *"Deck(brand)sohle"* write two compounds at once, and the head after the
    /// closing bracket is as lower case as the fragment after a hyphen. So is
    /// the rest of a word behind a gender star: *"Leser*innenkommentar"*.
    fn is_hyphen_compound_fragment(
        token: &Token,
        prev: Option<&Token>,
        next: Option<&Token>,
    ) -> bool {
        let hyphen = |t: &Token| matches!(t.kind, TokenKind::Punctuation(Punctuation::Hyphen));
        let infix_close =
            |t: &Token| matches!(t.kind, TokenKind::Punctuation(Punctuation::CloseRound));
        let gender_star = |t: &Token| matches!(t.kind, TokenKind::Punctuation(Punctuation::Star));

        prev.is_some_and(|p| {
            (hyphen(p) || infix_close(p) || gender_star(p)) && p.span.end == token.span.start
        }) || next.is_some_and(|n| hyphen(n) && token.span.end == n.span.start)
    }

    /// Is this token inside a foreign-language gloss?
    ///
    /// Walks left over the comma-separated list a language name introduces —
    /// *"althochdeutsch reht, recht, rehd, riht, reth"* — and stops at the first
    /// token that cannot be part of one. Function words end the gloss, so
    /// *"Er lernt englisch und geht in die stadt"* still flags `stadt`.
    fn follows_language_gloss(tokens: &[&Token], index: usize, document: &Document) -> bool {
        let mut i = index;
        for _ in 0..8 {
            if i == 0 {
                return false;
            }
            i -= 1;
            let token = tokens[i];

            if matches!(token.kind, TokenKind::Punctuation(Punctuation::Comma)) {
                continue;
            }
            if !matches!(token.kind, TokenKind::Word(_)) {
                return false;
            }

            let lower = noun_phrase::lowercase_of(token, document);
            if LANGUAGE_GLOSS_MARKERS.contains(&lower.as_str()) {
                return true;
            }
            // Only the quoted forms themselves may stand between the marker and
            // this token; a determiner, preposition or conjunction ends it.
            if token.kind.is_determiner()
                || token.kind.is_preposition()
                || token.kind.is_pronoun()
                || token.kind.is_conjunction()
                || noun_phrase::is_lowercase_non_noun(&lower)
            {
                return false;
            }
        }
        false
    }

    /// Does the dictionary know `word` as an adjective?
    fn is_adjective(&self, word: &[char]) -> bool {
        self.dictionary
            .get_word_metadata(word)
            .is_some_and(|m| m.adjective.is_some())
    }

    /// Is `lower` an adjective carrying a declension ending?
    ///
    /// Only the bare `-e` ending is handled here; `-en`, `-em`, `-er` and `-es`
    /// are already rejected wholesale further up. Comparatives decline on top of
    /// their own `-er` ("genau" → "genauer" → "genauere"), so that layer is
    /// peeled off too.
    ///
    /// The cost is a handful of nominalized adjectives that really are nouns —
    /// "die Breite", "die Tiefe" — which this no longer flags when written lower
    /// case. Attributive adjectives outnumber them heavily, and a missed lint is
    /// the cheaper mistake.
    fn is_declined_adjective(&self, lower: &[char]) -> bool {
        let Some(stem) = lower.strip_suffix(&['e']) else {
            return false;
        };

        if stem.len() < 3 {
            return false;
        }

        if self.is_adjective(stem) {
            return true;
        }

        stem.strip_suffix(&['e', 'r'])
            .is_some_and(|positive| positive.len() >= 3 && self.is_adjective(positive))
    }

    /// Is `lower` a present participle — a verb stem plus `-d`?
    ///
    /// German builds it from the infinitive: "liegen" → "liegend", "handeln" →
    /// "handelnd", "fortdauern" → "fortdauernd". Asking the dictionary for the
    /// infinitive keeps nouns that merely end the same way ("Abend", "Jugend",
    /// "Tugend") out of it, which a bare suffix test cannot do.
    fn is_present_participle(&self, lower: &[char]) -> bool {
        let Some(infinitive) = lower.strip_suffix(&['d']) else {
            return false;
        };

        if infinitive.len() < 4
            || !(infinitive.ends_with(&['e', 'n'])
                || infinitive.ends_with(&['e', 'l', 'n'])
                || infinitive.ends_with(&['e', 'r', 'n']))
        {
            return false;
        }

        self.dictionary
            .get_word_metadata(infinitive)
            .is_some_and(|m| m.verb.is_some())
    }

    /// Decide whether a lowercase, alphabetic, non-sentence-initial word should
    /// be flagged as a miscapitalized noun.
    fn check_if_word_is_noun(
        &self,
        word_chars: &[char],
        prev: Option<&Token>,
        np_role: noun_phrase::Role,
        verb_second: bool,
    ) -> bool {
        let lower: Vec<char> = word_chars
            .iter()
            .map(|c| c.to_lowercase().next().unwrap_or(*c))
            .collect();
        let s: String = lower.iter().collect();
        let nchars = lower.len();

        if nchars < 2 {
            return false;
        }

        // Foreign etymology terms (téchnē, lógos, eurýs, ʕarab, ...) carry
        // letters outside the German alphabet. They are quoted Latin/Greek, not
        // miscapitalized German nouns.
        if !lower
            .iter()
            .all(|c| c.is_ascii_lowercase() || matches!(c, 'ä' | 'ö' | 'ü' | 'ß'))
        {
            return false;
        }

        // Hard non-noun classes.
        if Self::is_non_noun(&lower)
            || NUMERALS.contains(&s)
            || UNIT_ABBREVIATIONS.contains(&s)
            || FOREIGN_TERMS.contains(&s)
        {
            return false;
        }

        // A number immediately to the left → unit or list item ("45 km",
        // "Forderung 2 weg"), not a noun.
        if let Some(p) = prev
            && matches!(p.kind, TokenKind::Number(_) | TokenKind::Decade)
        {
            return false;
        }

        // Adjective / adverb / participle shape → not a noun, even if the
        // compound-aware dictionary decomposed it into one.
        if has_non_noun_ending(&s) {
            return false;
        }

        // Declined adjectives ("die britische Krone") and present participles
        // ("in Führung liegend ausgeschieden") are written lower case and sit in
        // exactly the slot a noun would. Neither is in the dictionary as its own
        // entry, so both arrive here carrying a spurious noun reading picked up
        // from a plural or genitive flag. The stem gives them away, and the
        // dictionary already knows it.
        //
        // Not in head position, though: there the very same forms are genuine
        // nominalizations that *should* be flagged — "auf das wesentliche", "nur
        // für deutsche". Head is decided below, on syntax rather than shape.
        if np_role != noun_phrase::Role::Head
            && (self.is_declined_adjective(&lower) || self.is_present_participle(&lower))
        {
            return false;
        }

        // Separable-prefix finite verb forms (herausrückt, zurückgeht, ...).
        if SEPARABLE_VERB_PREFIXES
            .iter()
            .any(|p| s.starts_with(p) && nchars > p.chars().count() + 2)
            && (s.ends_with('t') || s.ends_with("te") || s.ends_with("st") || s.ends_with('n'))
        {
            return false;
        }

        // Verb-form shape (infinitive / conjugated): "-en/-eln/-ern", "-est",
        // "-et", "-te", "-ten". Also inflected-adjective / plural shape "-er",
        // "-es", "-em". None of the strong noun suffixes below end this way, so
        // this is a safe blanket reject and matches the rule's historical
        // false-negative profile (lowercase "-en" plurals are not chased).
        if nchars > 3
            && (s.ends_with("en")
                || s.ends_with("eln")
                || s.ends_with("ern")
                || s.ends_with("est")
                || s.ends_with("et")
                || s.ends_with("te")
                || s.ends_with("ten")
                || s.ends_with("er")
                || s.ends_with("es")
                || s.ends_with("em"))
        {
            return false;
        }

        let word_meta = self.dictionary.get_word_metadata(word_chars);
        let lower_meta = self.dictionary.get_word_metadata(&lower);

        let any = |f: &dyn Fn(&crate::DictWordMetadata) -> bool| -> bool {
            if let Some(m) = word_meta.as_deref()
                && f(m)
            {
                return true;
            }
            if let Some(m) = lower_meta.as_deref()
                && f(m)
            {
                return true;
            }
            false
        };

        let has_noun = any(&|m| m.noun.is_some());
        let has_verb = any(&|m| m.verb.is_some());
        let has_adjective = any(&|m| m.adjective.is_some());
        let has_adverb = any(&|m| m.adverb.is_some());
        let has_closed_class = any(&|m| {
            m.pronoun.is_some()
                || m.determiner.is_some()
                || m.conjunction.is_some()
                || m.preposition
        });

        // A recognized derivational noun suffix (-ung, -heit, -keit, -schaft,
        // -tät, -ion, -nis, -tum, -ling) is a near-certain noun. This is meant
        // to catch nouns the dictionary is missing entirely, so only trust it
        // for out-of-vocabulary words or ones the dictionary already calls a
        // noun — not for entries deliberately tagged POS-neutral (Latin terms
        // like "terminis" that merely happen to end in "-nis").
        let in_dictionary = word_meta.is_some() || lower_meta.is_some();
        let has_strong_noun_suffix = self
            .noun_suffixes
            .iter()
            .any(|(suffix, min_len)| nchars >= *min_len && lower.ends_with(suffix.as_slice()));
        if has_strong_noun_suffix && !has_verb && (!in_dictionary || has_noun) {
            return true;
        }

        if !has_noun || has_closed_class {
            return false;
        }

        // Bare "-e": genuine feminine/neuter nouns (Blume, Sonne, Frage) carry
        // gender, or form a plural and say so; 1st-person verb forms and
        // inflected adjectives do neither.
        //
        // The plural is the half that has to be named explicitly. This asked
        // for any agreement feature at all until nouns whose plural this
        // dictionary cannot build started carrying a bare singular, at which
        // point `file`, `single`, `hardware`, `grace` and `zuhause` all passed
        // it. A singular on its own says only "this is a noun somewhere", which
        // was never the question.
        if s.ends_with('e') {
            let gendered = any(&|m| {
                m.is_noun()
                    && (!m.noun_agreement().gender.is_empty()
                        || m.noun_agreement().number.contains(NumberSet::PLURAL))
            });
            if !gendered {
                return false;
            }
        }

        let ambiguous = has_verb || has_adjective || has_adverb;

        if !ambiguous {
            // Clean, unambiguous noun reading (noun, but no verb / adjective /
            // adverb reading): flag it wherever it appears.
            return true;
        }

        // Ambiguous noun / verb (or noun / adjective) homograph: a noun here
        // only if it is the *head* of a noun phrase. As a modifier it is the
        // attributive adjective ("die wesentliche Frage"), and outside a noun
        // phrase it is the verb ("..., fang an"). And a head that can be a
        // verb, behind a clause-initial pronoun, is the clause's verb — see
        // `is_verb_second`.
        matches!(np_role, noun_phrase::Role::Head) && !(has_verb && verb_second)
    }
}

impl<T: Dictionary> Linter for GermanNounCapitalization<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for paragraph in document.iter_paragraphs() {
            for sentence in paragraph.iter_sentences() {
                let first_word_span = sentence.first_non_whitespace().map(|t| t.span);
                let tokens: Vec<&Token> = sentence
                    .iter()
                    .filter(|t| !t.kind.is_whitespace())
                    .collect();
                let np_roles = noun_phrase::roles(&tokens, document);

                for (i, token) in tokens.iter().enumerate() {
                    if !matches!(token.kind, TokenKind::Word(_)) {
                        continue;
                    }

                    let word_chars = document.get_span_content(&token.span);
                    let prev = i.checked_sub(1).map(|j| tokens[j]);
                    let next = tokens.get(i + 1).copied();

                    if Self::is_hyphen_compound_fragment(token, prev, next)
                        || Self::follows_language_gloss(&tokens, i, document)
                        || Self::in_foreign_stretch(&tokens, i, document)
                        || Self::follows_subject_pronoun(prev, document)
                        || Self::is_predicative_noun(&tokens, i, document)
                    {
                        continue;
                    }

                    let already_capitalized = word_chars
                        .first()
                        .is_some_and(|first_char| first_char.is_uppercase());
                    let all_alphabetic = word_chars.iter().all(|c| c.is_alphabetic());
                    // The first word of a sentence is handled by
                    // `GermanSentenceCapitalization`; noun-vs-verb cannot be
                    // told apart there anyway ("Fang an!").
                    let is_sentence_initial = Some(token.span) == first_word_span;

                    if !already_capitalized
                        && all_alphabetic
                        && !is_sentence_initial
                        && self.check_if_word_is_noun(
                            word_chars,
                            prev,
                            np_roles[i],
                            Self::is_verb_second(&tokens, i, document),
                        )
                    {
                        let mut replacement: Vec<char> = word_chars.to_vec();
                        if let Some(first_char) = replacement.first_mut() {
                            *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
                        }

                        lints.push(Lint {
                            span: token.span,
                            lint_kind: LintKind::Capitalization,
                            suggestions: vec![Suggestion::ReplaceWith(replacement)],
                            priority: 25, // High priority for German
                            message: format!(
                                "Nomen werden im Deutschen großgeschrieben. »{}« ist offenbar ein Nomen.",
                                word_chars.iter().collect::<String>()
                            ),
                        });
                    }
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Achtet darauf, dass Nomen großgeschrieben werden."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use crate::language::german::spell::combined_german_dictionary;

    fn test_linter() -> GermanNounCapitalization<impl Dictionary> {
        GermanNounCapitalization::new(combined_german_dictionary())
    }

    fn create_document(text: &str) -> Document {
        Document::new_markdown_default(text, &combined_german_dictionary())
    }

    #[test]
    fn test_nouns_are_detected() {
        let mut linter = test_linter();
        let text = "die mondlandung";
        let document = create_document(text);
        let lints = linter.lint(&document);

        // "mondlandung" should be detected as a noun and flagged for capitalization
        assert!(
            !lints.is_empty(),
            "Expected at least one lint for lowercase noun"
        );
        let lint = &lints[0];
        let word: String = document.get_span_content(&lint.span).iter().collect();
        assert_eq!(word, "mondlandung");
        assert!(lint.message.contains("Nomen"));
    }

    #[test]
    fn test_simple_nouns_are_detected() {
        let mut linter = test_linter();
        let text = "der mond ist aufgegangen";
        let document = create_document(text);
        let lints = linter.lint(&document);

        // "mond" should be detected as a noun and flagged for capitalization
        assert!(
            !lints.is_empty(),
            "Expected at least one lint for lowercase noun 'mond'"
        );
        let lint = &lints[0];
        let word: String = document.get_span_content(&lint.span).iter().collect();
        assert_eq!(word, "mond");
        assert!(lint.message.contains("Nomen"));
    }

    #[test]
    fn test_verbs_are_not_detected_as_nouns() {
        let mut linter = test_linter();
        let text = "ich schreibe und lerne";
        let document = create_document(text);
        let lints = linter.lint(&document);

        // "schreibe" and "lerne" should NOT be detected as nouns
        assert_eq!(lints.len(), 0, "Verbs should not be detected as nouns");
    }

    #[test]
    fn test_past_participles_are_not_detected_as_nouns() {
        let mut linter = test_linter();
        let text = "es ist fehlgeschlagen";
        let document = create_document(text);
        let lints = linter.lint(&document);

        // "fehlgeschlagen" should NOT be detected as a noun
        assert_eq!(
            lints.len(),
            0,
            "Past participles should not be detected as nouns"
        );
    }

    #[test]
    fn test_noun_suffixes_still_work() {
        let mut linter = test_linter();
        let text = "die freiheit und die menschheit";
        let document = create_document(text);
        let lints = linter.lint(&document);

        // "freiheit" and "menschheit" should be detected as nouns via suffix
        assert!(
            !lints.is_empty(),
            "Expected at least one lint for nouns with suffixes"
        );
    }

    #[test]
    fn test_mixed_nouns_and_verbs() {
        let mut linter = test_linter();
        let text = "die mondlandung ist wieder fehlgeschlagen";
        let document = create_document(text);
        let lints = linter.lint(&document);

        // Only "mondlandung" should be detected as a noun
        assert_eq!(
            lints.len(),
            1,
            "Expected exactly one lint for 'mondlandung'"
        );
        let lint = &lints[0];
        let word: String = document.get_span_content(&lint.span).iter().collect();
        assert_eq!(word, "mondlandung");
    }

    #[test]
    fn test_noun_verb_homograph_uses_context() {
        let mut linter = test_linter();

        // Licensed by the article "der" -> noun -> flag.
        let doc = create_document("der fang ist groß");
        let lints = linter.lint(&doc);
        assert_eq!(
            lints.len(),
            1,
            "\"der fang\" should be flagged as a noun ({:?})",
            lints
                .iter()
                .map(|l| document_word(&doc, l))
                .collect::<Vec<_>>()
        );
        assert_eq!(document_word(&doc, &lints[0]), "fang");

        // Not licensed (imperative after a comma) -> verb -> no flag.
        let doc = create_document("ich sage dir, fang an");
        let lints = linter.lint(&doc);
        assert_eq!(
            lints.len(),
            0,
            "\"..., fang an\" should not be flagged ({:?})",
            lints
                .iter()
                .map(|l| document_word(&doc, l))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_numbers_and_units_are_not_flagged() {
        let mut linter = test_linter();
        let doc = create_document("Die Strecke ist etwa 45 km lang und hat drei Abschnitte");
        let lints = linter.lint(&doc);
        let flagged: Vec<String> = lints.iter().map(|l| document_word(&doc, l)).collect();
        assert!(
            !flagged.iter().any(|w| w == "km" || w == "drei"),
            "units and number words should not be flagged, got {flagged:?}"
        );
    }

    #[test]
    fn test_bracketed_compound_heads_are_not_flagged() {
        let mut linter = test_linter();
        for text in [
            "Die Trippe kann als Holz(über)schuh angesprochen werden.",
            "Die Innensohle ist durch eine Deck(brand)sohle abgedeckt.",
        ] {
            let doc = create_document(text);
            assert!(linter.lint(&doc).is_empty(), "should not fire on {text:?}");
        }

        let doc = create_document("Bitte als Leser*innenkommentar hinterlassen.");
        assert!(linter.lint(&doc).is_empty(), "a gender star joins one word");

        let doc = create_document("Er trägt (neue) schuhe.");
        assert_eq!(
            linter.lint(&doc).len(),
            1,
            "a spaced bracket ends no compound"
        );
    }

    fn document_word(document: &Document, lint: &Lint) -> String {
        document.get_span_content(&lint.span).iter().collect()
    }
}
