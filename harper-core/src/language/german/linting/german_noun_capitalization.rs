use crate::{
    Punctuation, Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::spell::lexical_classes::{FOREIGN_TERMS, NUMERALS, UNIT_ABBREVIATIONS},
    language::morphology::MorphologyExt,
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

/// Words whose lower-case reading is not the noun one.
///
/// This used to be 265 words, and the reason given was that "the dictionary
/// actively mistags them". That is no longer true:
/// `scripts/strip_german_noun_readings.py` took the noun reading off every
/// lower-case entry igerman98 has no capitalized form for, and 230 of these
/// words stopped reading as nouns with it.
///
/// What is left is the part a dictionary cannot settle. Each of these really is
/// a noun when it is capitalized -- `die Frage`, `die Waren`, `das Gut`, `die
/// Wegen` -- so the entry is right to carry the reading, and the lower-case
/// occurrence still has to be let through. A handful are not entries at all.
///
/// Check the list against the dictionary before adding to it:
///
/// ```bash
/// just language-meta-text german "wegen trotz ist waren"
/// ```
const GERMAN_NON_NOUNS: &[&str] = &[
    "alt", "arbeite", "darin", "denke", "dgl", "dürfen", "ebd", "etc", "frage", "gebe", "gibe",
    "groß", "gut", "habe", "heute", "hin", "ist", "klein", "kurz", "können", "lang", "langsam",
    "neu", "sehe", "sollen", "sondern", "teils", "trotz", "versuche", "viel", "waren", "wegen",
    "wollen", "worden", "wäre",
];

// The spelled-out numerals, unit abbreviations and lower-case foreign terms
// that used to live here are now dictionary data, carried by the property flags
// `1`, `2` and `3` in `dictionary.dict` / `annotations.json`. They are read back
// as sets by `spell::lexical_classes`; adding a word no longer means editing Rust.

/// Words that, standing immediately to the left of a candidate, mark it as the
/// head or a modifier of a noun phrase: articles, other determiners,
/// possessives, demonstratives, quantifiers and the common prepositions
/// (including the usual contracted forms). Kept as an explicit surface list
/// because the German dictionary mislabels many of these forms.
const NOUN_PHRASE_LICENSORS: &[&str] = &[
    // definite / indefinite articles, all cases
    "der",
    "die",
    "das",
    "dem",
    "den",
    "des",
    "ein",
    "eine",
    "einen",
    "einem",
    "einer",
    "eines",
    "kein",
    "keine",
    "keinen",
    "keinem",
    "keiner",
    "keines",
    // possessives
    "mein",
    "meine",
    "meinen",
    "meinem",
    "meiner",
    "meines",
    "dein",
    "deine",
    "deinen",
    "deinem",
    "deiner",
    "deines",
    "sein",
    "seine",
    "seinen",
    "seinem",
    "seiner",
    "seines",
    "ihr",
    "ihre",
    "ihren",
    "ihrem",
    "ihrer",
    "ihres",
    "unser",
    "unsere",
    "unseren",
    "unserem",
    "unserer",
    "unseres",
    "euer",
    "eure",
    "euren",
    "eurem",
    "eurer",
    "eures",
    // demonstratives / relatives / quantifiers
    "dies",
    "dieser",
    "diese",
    "dieses",
    "diesen",
    "diesem",
    "jen",
    "jener",
    "jene",
    "jenes",
    "jenen",
    "jenem",
    "jed",
    "jeder",
    "jede",
    "jedes",
    "jeden",
    "jedem",
    "manch",
    "mancher",
    "manche",
    "manches",
    "manchen",
    "manchem",
    "solch",
    "solcher",
    "solche",
    "solches",
    "solchen",
    "solchem",
    "welch",
    "welcher",
    "welche",
    "welches",
    "welchen",
    "welchem",
    "all",
    "alle",
    "allen",
    "aller",
    "alles",
    "allem",
    "beide",
    "beiden",
    "beider",
    "sämtliche",
    "sämtlichen",
    "jegliche",
    "jeglichen",
    "mehrere",
    "mehreren",
    "einige",
    "einigen",
    "einiger",
    "viele",
    "vielen",
    "wenige",
    "wenigen",
    // prepositions fused with an article: these *are* a determiner ("im
    // Freien", "zum Guten"), which is why they are on this side of the split.
    "im",
    "ins",
    "am",
    "ans",
    "aufs",
    "beim",
    "vom",
    "zum",
    "zur",
    "übers",
    "unters",
    "durchs",
    "fürs",
    "ums",
];

/// Prepositions that open a noun phrase without supplying a determiner.
///
/// German nominalizes an adjective only under a determiner, and the nominalized
/// form always carries a declension ending: *das Braune*, *im Freien*, *ein
/// Kahler*. A bare preposition in front of a **base-form** adjective is
/// therefore never a nominalization — *"weiß bis braun"*, *"von gelb zu weiß"*
/// are predicative. Kept apart from [`NOUN_PHRASE_LICENSORS`] for exactly that
/// distinction; both open a phrase.
const NP_BARE_PREPOSITIONS: &[&str] = &[
    "in",
    "an",
    "auf",
    "aus",
    "bei",
    "mit",
    "nach",
    "von",
    "vor",
    "zu",
    "über",
    "unter",
    "durch",
    "für",
    "gegen",
    "ohne",
    "um",
    "seit",
    "während",
    "wegen",
    "trotz",
    "statt",
    "anstatt",
    "innerhalb",
    "außerhalb",
    "oberhalb",
    "unterhalb",
    "entlang",
    "gegenüber",
    "bis",
    "per",
    "pro",
    "via",
    "samt",
    "nebst",
    "laut",
    "gemäß",
    "mittels",
    "anhand",
    "aufgrund",
    "infolge",
    "zwischen",
    "neben",
    "hinter",
];

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

/// Words that join coordinated attributive adjectives inside one noun phrase:
/// *"in britische, französische **und** niederländische Kolonien"*.
/// Function words of the languages German quotes without translating, chosen so
/// that **none of them is also a German word**. `des`, `in`, `da`, `so` and `e`
/// are deliberately absent for that reason, even though they are frequent in
/// French, Italian and Latin.
const FOREIGN_FUNCTION_WORDS: &[&str] = &[
    // English
    "the", "of", "and", "for", "with", "from", "their", "its", "his", "her", "they", "these",
    "those", "which", "that", "about", "into", "upon", "between", "among", "were", "been", "being",
    "there", "when", "where", "would", "could", "should", // French
    "la", "le", "les", "du", "de", "au", "aux", "et", "une", "dans", "sur", "pour", "avec", "sans",
    "chez", "leur", "ses", "son", "sa", "cette", "ces", "qui", "que",
    // Italian / Spanish / Portuguese
    "della", "delle", "degli", "dei", "nel", "nella", "il", "lo", "gli", "una", "col", "por",
    "para", "los", "las", "del", "el", "uma", "dos", // Latin
    "apud", "atque", "quae", "quod", "cum", "sive", "seu", "ratione", "liber", "libri",
];

/// Forms that are a relative pronoun as readily as an article or determiner.
/// Only [`GermanNounCapitalization::opens_relative_clause`] uses this, and only
/// straight after a comma or an opening bracket.
const RELATIVE_PRONOUNS: &[&str] = &[
    "der", "die", "das", "dem", "den", "dessen", "deren", "denen", "welcher", "welche", "welches",
    "welchen", "welchem", "wer", "wen", "wem", "was",
];

const COORDINATORS: &[&str] = &[
    "und",
    "oder",
    "sowie",
    "beziehungsweise",
    "bzw",
    // Contrastive: "der milde, **aber** wenig angenehme Pilz". They join two
    // attributive adjectives exactly the way "und" does, and stopping on one
    // promoted the adjective in front of it to head.
    "aber",
    "jedoch",
    "sondern",
];

/// Degree words that grade the adjective following them. They stand inside a
/// noun phrase without being a modifier or the head of it.
const DEGREE_MODIFIERS: &[&str] = &[
    "etwas",
    "sehr",
    "ziemlich",
    "recht",
    "besonders",
    "eher",
    "noch",
    "weit",
    "weitaus",
    "deutlich",
    "leicht",
    "kaum",
    "durchaus",
    "überaus",
    "äußerst",
    "höchst",
    "relativ",
    "vergleichsweise",
    "zunehmend",
    "teilweise",
    "meist",
    "vorwiegend",
    "überwiegend",
    // Quantity words used as degree: "der milde, aber **wenig** angenehme Pilz".
    "wenig",
    "viel",
    "fast",
    "nahezu",
    "annähernd",
    "ausgesprochen",
    "vorwiegend",
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
        GERMAN_NON_NOUNS.contains(&s.as_str())
    }

    /// Does this token open a noun phrase — an article, another determiner, a
    /// possessive, a preposition or a spelled-out number?
    fn opens_noun_phrase(token: &Token, document: &Document) -> bool {
        // Punctuation, numbers, whitespace and symbols never open a noun
        // phrase. Only a genuine word can.
        if !matches!(token.kind, TokenKind::Word(_)) {
            return false;
        }

        if token.kind.is_preposition() || token.kind.is_determiner() {
            return true;
        }

        let lower = Self::lowercase_of(token, document);
        NOUN_PHRASE_LICENSORS.contains(&lower.as_str())
            || NP_BARE_PREPOSITIONS.contains(&lower.as_str())
            || NUMERALS.contains(&lower)
    }

    /// Does this token supply a **determiner**, rather than merely open a phrase?
    ///
    /// The distinction only matters for nominalized adjectives, which German
    /// licenses with a determiner and writes with a declension ending: *das
    /// Braune*, *im Freien*. A bare preposition or a numeral supplies neither, so
    /// *"weiß bis braun"*, *"von gelb zu weiß"* and *"davon sind vier unbewohnt"*
    /// are predicative adjectives — not noun phrases whose head happens to be
    /// lower case.
    /// Is this adjective in its undeclined base form?
    ///
    /// German nominalizes an adjective *with* a declension ending — "für
    /// **Deutsche**", "das **Gute**", "im **Freien**" — so a declined form after
    /// a bare preposition is a real nominalization and stays a candidate. The
    /// base form never is one: "weiß bis **braun**" and "von **gelb** zu weiß"
    /// are predicative, and the noun spelling would be a separate lexeme ("das
    /// Braun") rather than this word.
    fn is_base_form_adjective(token: &Token, document: &Document) -> bool {
        let lower = Self::lowercase_of(token, document);
        !(lower.ends_with('e')
            || lower.ends_with("en")
            || lower.ends_with("er")
            || lower.ends_with("es")
            || lower.ends_with("em"))
    }

    fn supplies_determiner(token: &Token, document: &Document) -> bool {
        if !matches!(token.kind, TokenKind::Word(_)) {
            return false;
        }
        if token.kind.is_determiner() {
            return true;
        }
        NOUN_PHRASE_LICENSORS.contains(&Self::lowercase_of(token, document).as_str())
    }

    /// Can this token sit *inside* a noun phrase — as an attributive adjective,
    /// as the head noun, or as an unknown word standing in for one?
    ///
    /// Deliberately reads only the metadata already attached to the token by
    /// `Document::parse`. Calling `Dictionary::get_word_metadata` once per token
    /// would take `CompoundAwareDictionary`'s global mutex and attempt a
    /// compound decomposition on every miss.
    fn continues_noun_phrase(token: &Token, document: &Document) -> bool {
        if !matches!(token.kind, TokenKind::Word(_)) {
            return false;
        }

        let chars = document.get_span_content(&token.span);
        if chars.is_empty() || !chars.iter().all(|c| c.is_alphabetic()) {
            return false;
        }

        // Closed-class words close the phrase: "die Zeit **im** Büro" is two
        // noun phrases, not one.
        if token.kind.is_determiner()
            || token.kind.is_preposition()
            || token.kind.is_pronoun()
            || token.kind.is_conjunction()
        {
            return false;
        }

        // A capital letter mid-sentence marks the head noun (or a proper name),
        // and it wins over every part-of-speech reading below. The dictionary
        // hands out spurious adverb and verb readings freely — "Band" is tagged
        // an adverb — and rejecting the token on one of those truncates the
        // phrase and promotes the attributive adjective before it to head.
        // `GERMAN_NON_NOUNS` suppresses lints on *lowercase* verb forms, several
        // of which are perfectly good nouns when written with a capital — "die
        // Frage", "die Sage", "die Suche".
        if chars.first().is_some_and(|c| c.is_uppercase()) {
            return true;
        }

        // Adverbs close the phrase. Only adjectives stand between the determiner
        // and the head, so "das Thema **oftmals** behandelt" ends the phrase at
        // "Thema" instead of making the adverb its head. Words that are both —
        // most German adjectives double as adverbs — keep going.
        if token.kind.is_adverb() && !token.kind.is_adjective() {
            return false;
        }

        let lower = Self::lowercase_of(token, document);
        if GERMAN_NON_NOUNS.contains(&lower.as_str()) {
            return false;
        }

        // An adjective reading marks an attributive modifier, a noun reading a
        // (miscapitalized) head. An out-of-vocabulary word is most likely one of
        // the two, and treating it as phrase-internal keeps the head from being
        // mistaken for the word before it. So is a word the dictionary lists
        // with no part of speech at all — `diversifizierte` is in there carrying
        // nothing, and ending the phrase on it made "eine **reiche** und
        // diversifizierte Tierwelt" a capitalization error.
        token.kind.is_noun()
            || token.kind.is_adjective()
            || token.kind.is_oov()
            || Self::has_no_pos_reading(token)
    }

    /// Is the token a word the dictionary knows but gives no part of speech?
    ///
    /// Distinct from [`TokenKind::is_oov`], which is the *missing* entry. An
    /// entry with every reading empty carries no evidence either way, and the
    /// chunker has to treat it the same as an unknown word rather than as a
    /// phrase boundary.
    fn has_no_pos_reading(token: &Token) -> bool {
        match &token.kind {
            TokenKind::Word(Some(metadata)) => {
                metadata.noun.is_none()
                    && metadata.verb.is_none()
                    && metadata.adjective.is_none()
                    && metadata.adverb.is_none()
                    && metadata.pronoun.is_none()
                    && metadata.determiner.is_none()
                    && metadata.conjunction.is_none()
                    && !metadata.preposition
            }
            _ => false,
        }
    }

    /// Is the token at `index` a relative pronoun rather than an article?
    ///
    /// German spells them the same — `der`, `die`, `das`, `dem`, `den` are both
    /// — and the difference decides what follows: an article introduces a noun
    /// phrase, a relative pronoun a verb-final clause. The comma is the reliable
    /// surface signal, because German punctuates every relative clause:
    ///
    /// ```text
    /// der SV Rödinghausen, der zuletzt 2019 den Pokal gewinnen konnte
    ///                      ^relative pronoun — "zuletzt" is not its head noun
    /// ```
    ///
    /// Reading it as an article crowns whatever comes next, which in a relative
    /// clause is an adverb or a finite verb: `unterging`, `angibt`, `verstreut`
    /// and `zuletzt` were all reported as miscapitalized nouns this way.
    ///
    /// The cost is a lint inside *"das Haus, das große fenster hat"* — a noun
    /// phrase really does follow there. It stays unflagged, which is the same
    /// trade the rule makes everywhere else: a missed lint over a false one.
    fn opens_relative_clause(tokens: &[&Token], index: usize, document: &Document) -> bool {
        if !RELATIVE_PRONOUNS.contains(&Self::lowercase_of(tokens[index], document).as_str()) {
            return false;
        }

        index
            .checked_sub(1)
            .map(|previous| tokens[previous])
            .is_some_and(|previous| {
                matches!(
                    previous.kind,
                    TokenKind::Punctuation(Punctuation::Comma | Punctuation::OpenRound)
                )
            })
    }

    fn lowercase_of(token: &Token, document: &Document) -> String {
        document
            .get_span_content(&token.span)
            .iter()
            .map(|c| c.to_lowercase().next().unwrap_or(*c))
            .collect()
    }

    /// Chunk a sentence into noun phrases and label each token's role.
    ///
    /// German noun phrases are rigid — determiner/preposition, then any number
    /// of attributive adjectives, then the head noun — which is why a rule can
    /// do here what English needs a trained chunker (`DictWordMetadata::np_member`)
    /// for. Only the **head** of a phrase is a candidate for capitalization:
    ///
    /// ```text
    /// die   wesentliche   Frage      der  große     schöne     hund
    /// ^open ^modifier     ^head      ^open ^modifier ^modifier  ^head
    /// ```
    ///
    /// Looking only at the token to the left, as this linter used to, flags
    /// `wesentliche` in the first phrase and both `große` and `schöne` in the
    /// second, while missing `hund` — the word that actually needs a capital.
    ///
    /// The head is always **last**: everything before it is an attributive
    /// adjective, which German writes lower case, and the head itself is
    /// capitalized unless it is the very error being looked for. So a
    /// capitalized token ends the phrase — it *is* the head. Without that,
    /// *"in Munitionsfabriken eingesetzt"* runs on past `Munitionsfabriken` and
    /// makes the participle the head.
    fn noun_phrase_roles(tokens: &[&Token], document: &Document) -> Vec<NpRole> {
        let mut roles = vec![NpRole::Outside; tokens.len()];

        let mut i = 0;
        while i < tokens.len() {
            if !Self::opens_noun_phrase(tokens[i], document)
                || Self::opens_relative_clause(tokens, i, document)
            {
                i += 1;
                continue;
            }

            let mut end = i + 1;
            loop {
                if end >= tokens.len() {
                    break;
                }

                // Coordinated attributive adjectives are joined by a comma or a
                // conjunction: "eine neue, radikalere Welle", "in britische,
                // französische und niederländische Kolonien". Step over the
                // joiner so the adjectives stay modifiers instead of each
                // becoming a phrase of its own.
                let joins_coordination =
                    matches!(tokens[end].kind, TokenKind::Punctuation(Punctuation::Comma))
                        || COORDINATORS
                            .contains(&Self::lowercase_of(tokens[end], document).as_str());

                // A degree word may sit in front of any of those adjectives:
                // "der gerade oder **etwas** gekrümmte Griffel". It is neither a
                // modifier nor the head, but stopping on it would leave the
                // adjective before it standing as the head.
                let grades_next_adjective =
                    DEGREE_MODIFIERS.contains(&Self::lowercase_of(tokens[end], document).as_str());

                // So can a numeral or an opening bracket or quote: "das
                // beginnende **19.** Jahrhundert", "die britische
                // **4x100-Meter-**Mannschaft", "eine eigene **„**Baumnorm“",
                // "die deutsche **(**Wieder-)Besiedlung".
                let interrupts_phrase = matches!(
                    tokens[end].kind,
                    TokenKind::Number(_)
                        | TokenKind::Decade
                        | TokenKind::Punctuation(
                            Punctuation::Quote(_)
                                | Punctuation::OpenRound
                                | Punctuation::OpenSquare
                        )
                ) || Self::is_opening_mark(tokens[end], document);

                if (joins_coordination || grades_next_adjective || interrupts_phrase)
                    && end > i + 1
                    && Self::phrase_resumes_after(tokens, end, document)
                {
                    end += 1;
                    continue;
                }

                if !Self::continues_noun_phrase(tokens[end], document) {
                    break;
                }

                let capitalized = document
                    .get_span_content(&tokens[end].span)
                    .first()
                    .is_some_and(|c| c.is_uppercase());
                end += 1;
                if capitalized {
                    break;
                }
            }
            // A trailing joiner is not part of the phrase.
            while end > i + 1
                && (matches!(
                    tokens[end - 1].kind,
                    TokenKind::Punctuation(Punctuation::Comma)
                ) || COORDINATORS
                    .contains(&Self::lowercase_of(tokens[end - 1], document).as_str()))
            {
                end -= 1;
            }

            // A lone adjective under something that is not a determiner is
            // predicative, not a nominalization: "die Markzone ist weiß bis
            // **braun**", "von **gelb** zu weiß", "davon sind vier
            // **unbewohnt**". German needs a determiner for the nominal reading,
            // and then writes the declension ending with it — "das Braune". The
            // phrase is left headless rather than crowning the adjective.
            let predicative = end == i + 2
                && tokens[end - 1].kind.is_adjective()
                && Self::is_base_form_adjective(tokens[end - 1], document)
                && !Self::supplies_determiner(tokens[i], document);

            // An ordinal ends the *sentence* as far as the segmenter is
            // concerned, so "das sowjetische 170. | Regiment" arrives here cut in
            // half and the adjective is the last thing left. Everything after the
            // phrase being a numeral and a full stop is the signature of that
            // split; the head is in the next sentence, not on the adjective.
            let split_at_ordinal = tokens[end..].iter().all(|t| {
                matches!(
                    t.kind,
                    TokenKind::Number(_)
                        | TokenKind::Decade
                        | TokenKind::Punctuation(Punctuation::Period)
                )
            }) && tokens[end..]
                .iter()
                .any(|t| matches!(t.kind, TokenKind::Number(_) | TokenKind::Decade));

            if end > i + 1
                && !predicative
                && !(split_at_ordinal && tokens[end - 1].kind.is_adjective())
            {
                for role in roles.iter_mut().take(end - 1).skip(i + 1) {
                    *role = NpRole::Modifier;
                }
                roles[end - 1] = NpRole::Head;
            }

            // Resume at the phrase end; that token may itself open the next one.
            i = end.max(i + 1);
        }

        roles
    }

    /// Is this token inside a stretch of a foreign language?
    ///
    /// German prose quotes foreign titles without translating them, and a
    /// bibliography is mostly that: *"The Modes of scepticism: ancient **texts**
    /// and modern interpretations"*, *"Galien et la **philosophie**"*, *"Memorie
    /// della Reale Accademia **delle** Scienze"*. Several of those words are in
    /// the German dictionary with a noun reading — `texts`, `model`, `period`,
    /// `zone`, `roman` — so every one of them is reported as a lower-case German
    /// noun.
    ///
    /// The neighbourhood settles it. Two kinds of evidence count, within three
    /// word tokens on either side:
    ///
    /// * a function word no German sentence contains — `the`, `of`, `la`, `du`,
    ///   `et`, `della`. These are the strongest signal and the most common.
    /// * a word the dictionary does not know at all. Latin and taxonomic names
    ///   carry no function words — *"Conspectus generum avium"*, *"Mellisuga
    ///   minima vielloti"* — and are nothing but unknown words.
    ///
    /// At least one function word is **required**, and one more point has to
    /// come from somewhere — a second function word, or an unknown word.
    /// Unknown words alone are not enough and the difference is large: German
    /// Wikipedia is full of proper names the dictionary does not have, and
    /// letting two of those silence the rule cost a fifth of the injected
    /// lower-case nouns in `just language-recall german`.
    ///
    /// The price is the Latin and taxonomic runs, which carry no function word
    /// at all — *"Conspectus generum avium"* stays flagged. A handful of those
    /// against several hundred real errors is the right way round.
    fn in_foreign_stretch(tokens: &[&Token], index: usize, document: &Document) -> bool {
        const WINDOW: usize = 3;

        // A German determiner directly in front settles it the other way: *"durch
        // die Zeitschrift Le Mercure Galant"*, *"an der University of Virginia"*
        // are German sentences that happen to name something foreign, and the
        // word after the article is a German noun.
        if index
            .checked_sub(1)
            .is_some_and(|previous| Self::supplies_determiner(tokens[previous], document))
        {
            return false;
        }

        let mut function_words = 0;
        let mut unknown_words = 0;

        let mut adjacent_function_word = false;

        let mut visit = |token: &Token, distance: usize| {
            if !matches!(token.kind, TokenKind::Word(_)) {
                return;
            }
            if FOREIGN_FUNCTION_WORDS.contains(&Self::lowercase_of(token, document).as_str()) {
                function_words += 1;
                adjacent_function_word |= distance == 1;
            } else if token.kind.is_oov() {
                unknown_words += 1;
            }
        };

        for (offset, token) in tokens[..index].iter().rev().take(WINDOW).enumerate() {
            visit(token, offset + 1);
        }
        for (offset, token) in tokens.iter().skip(index + 1).take(WINDOW).enumerate() {
            visit(token, offset + 1);
        }

        function_words >= 2 || (adjacent_function_word && function_words + unknown_words >= 2)
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
    fn is_hyphen_compound_fragment(
        token: &Token,
        prev: Option<&Token>,
        next: Option<&Token>,
    ) -> bool {
        let hyphen = |t: &Token| matches!(t.kind, TokenKind::Punctuation(Punctuation::Hyphen));

        prev.is_some_and(|p| hyphen(p) && p.span.end == token.span.start)
            || next.is_some_and(|n| hyphen(n) && token.span.end == n.span.start)
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

            let lower = Self::lowercase_of(token, document);
            if LANGUAGE_GLOSS_MARKERS.contains(&lower.as_str()) {
                return true;
            }
            // Only the quoted forms themselves may stand between the marker and
            // this token; a determiner, preposition or conjunction ends it.
            if token.kind.is_determiner()
                || token.kind.is_preposition()
                || token.kind.is_pronoun()
                || token.kind.is_conjunction()
                || GERMAN_NON_NOUNS.contains(&lower.as_str())
            {
                return false;
            }
        }
        false
    }

    /// Is this token an opening quote or bracket, by its characters?
    ///
    /// The lexer is uneven about them: German's closing `“` arrives as
    /// `Punctuation::Quote`, but the opening `„` as `Unlintable`. Reading the
    /// character sidesteps the classification, and matching `Unlintable`
    /// wholesale is not an option — it is also what inline code blocks get, and
    /// those really do end a noun phrase.
    fn is_opening_mark(token: &Token, document: &Document) -> bool {
        let chars = document.get_span_content(&token.span);
        chars.len() == 1 && matches!(chars[0], '„' | '“' | '»' | '«' | '‚' | '‘' | '›' | '‹')
    }

    /// Does the phrase pick up again after the joiner or degree word at `index`?
    ///
    /// Several of them may stack — "eine große, aber **noch** **recht** junge
    /// Sammlung" — so the skip repeats until a token either continues the phrase
    /// or ends it.
    fn phrase_resumes_after(tokens: &[&Token], index: usize, document: &Document) -> bool {
        // Inside quotation marks a capitalized word is a title, whatever its
        // part of speech: *das neue „**Wir**“* is a noun phrase, not a pronoun
        // ending one.
        let quoted = Self::is_opening_mark(tokens[index], document);

        let mut next = index + 1;

        while next < tokens.len() {
            if quoted
                && matches!(tokens[next].kind, TokenKind::Word(_))
                && document
                    .get_span_content(&tokens[next].span)
                    .first()
                    .is_some_and(|c| c.is_uppercase())
            {
                return true;
            }

            if Self::continues_noun_phrase(tokens[next], document) {
                return true;
            }

            if !Self::skippable_inside_phrase(tokens[next], tokens.get(next - 1).copied(), document)
            {
                return false;
            }

            next += 1;
        }

        false
    }

    /// May this token stand between an attributive adjective and the head?
    ///
    /// German puts a surprising amount here: a grading adverb (*"eine große,
    /// aber **noch** recht junge Sammlung"*), a numeral or an ordinal (*"die
    /// ehemalige **84.** Oberschule"*, *"eine große, **1671** gefertigte Uhr"*,
    /// *"der lange **0,9 m** breite Gang"*), and further joiners when several
    /// stack (*"reich, **aber** **auch** vielfältig"*). None of them is a
    /// modifier or the head; stopping on one crowns the adjective in front of it
    /// and reports a capitalization error on a perfectly ordinary attributive.
    fn skippable_inside_phrase(token: &Token, prev: Option<&Token>, document: &Document) -> bool {
        if matches!(
            token.kind,
            TokenKind::Number(_) | TokenKind::Decade | TokenKind::Punctuation(Punctuation::Comma)
        ) {
            return true;
        }

        // The full stop of an ordinal, and only that one — a sentence-final
        // period is followed by a capitalized word, which `continues_noun_phrase`
        // accepts, so skipping it would run one phrase into the next sentence.
        if matches!(token.kind, TokenKind::Punctuation(Punctuation::Period))
            && prev.is_some_and(|p| matches!(p.kind, TokenKind::Number(_) | TokenKind::Decade))
        {
            return true;
        }

        let lower = Self::lowercase_of(token, document);
        DEGREE_MODIFIERS.contains(&lower.as_str())
            || COORDINATORS.contains(&lower.as_str())
            || UNIT_ABBREVIATIONS.contains(&lower)
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
        np_role: NpRole,
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
        if np_role != NpRole::Head
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
        // gender or number metadata; 1st-person verb forms and inflected
        // adjectives do not.
        if s.ends_with('e') {
            let gendered = any(&|m| m.is_noun() && m.has_noun_agreement());
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
        // phrase it is the verb ("..., fang an").
        matches!(np_role, NpRole::Head)
    }
}

/// A token's position in a German noun phrase, as labelled by
/// [`GermanNounCapitalization::noun_phrase_roles`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NpRole {
    /// The head of the phrase — the noun ("die wesentliche **Frage**").
    Head,
    /// An attributive modifier before the head ("die **wesentliche** Frage").
    Modifier,
    /// Not inside a noun phrase at all.
    Outside,
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
                let np_roles = Self::noun_phrase_roles(&tokens, document);

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
                        && self.check_if_word_is_noun(word_chars, prev, np_roles[i])
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
                                "In German, all nouns must be capitalized. \"{}\" appears to be a noun.",
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
        "Ensures German nouns are properly capitalized"
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
        assert!(lint.message.contains("noun"));
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
        assert!(lint.message.contains("noun"));
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

    fn document_word(document: &Document, lint: &Lint) -> String {
        document.get_span_content(&lint.span).iter().collect()
    }
}
