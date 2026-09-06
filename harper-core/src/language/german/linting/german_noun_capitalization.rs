use crate::{
    Token, TokenKind, TokenStringExt,
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

/// Common German function words that should never be flagged as nouns.
const GERMAN_NON_NOUNS: &[&str] = &[
    // Articles (all cases)
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
    // Pronouns
    "er",
    "sie",
    "es",
    "wir",
    "ihr",
    "ich",
    "du",
    "mich",
    "mir",
    "dich",
    "dir",
    "sich",
    "uns",
    "euch",
    "ihnen",
    "ihm",
    "dessen",
    "deren",
    "denen",
    "man",
    "wer",
    "wen",
    "wem",
    "was",
    // Possessives
    "mein",
    "dein",
    "sein",
    "unser",
    "euer",
    // Demonstratives / relative / quantifiers
    "dieser",
    "diese",
    "dieses",
    "diesen",
    "diesem",
    "jener",
    "jene",
    "jenes",
    "welch",
    "welcher",
    "welche",
    "welches",
    "welchen",
    "welchem",
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
    "solche",
    "solcher",
    "solches",
    "solchen",
    "solchem",
    "sämtliche",
    "sämtlichen",
    "jegliche",
    "jeglichen",
    "beide",
    "beiden",
    "beider",
    // Prepositions
    "in",
    "ins",
    "im",
    "an",
    "am",
    "auf",
    "aus",
    "bei",
    "mit",
    "nach",
    "von",
    "vor",
    "zu",
    "zum",
    "zur",
    "um",
    "für",
    "über",
    "unter",
    "zwischen",
    "neben",
    "hinter",
    "durch",
    "ohne",
    "gegen",
    "bis",
    "seit",
    "während",
    "wegen",
    "trotz",
    "statt",
    "außer",
    "ab",
    "ob",
    // Conjunctions
    "und",
    "oder",
    "aber",
    "denn",
    "weil",
    "dass",
    "wenn",
    "als",
    "ob",
    "sondern",
    "doch",
    "jedoch",
    "falls",
    "damit",
    "bevor",
    "nachdem",
    "obwohl",
    "während",
    "sobald",
    "solange",
    "sowie",
    "sowohl",
    "bzw",
    "usw",
    "etc",
    "vgl",
    "ebd",
    "dgl",
    // Adverbs
    "nicht",
    "auch",
    "noch",
    "schon",
    "wieder",
    "zudem",
    "nur",
    "sehr",
    "hier",
    "dort",
    "da",
    "darin",
    "davon",
    "dazu",
    "daran",
    "darauf",
    "dabei",
    "immer",
    "nie",
    "oft",
    "manchmal",
    "vielleicht",
    "wahrscheinlich",
    "heute",
    "morgen",
    "gestern",
    "jetzt",
    "dann",
    "so",
    "ganz",
    "gar",
    "heutzutage",
    "also",
    "fast",
    "bald",
    "her",
    "hin",
    "quasi",
    "zuerst",
    "zunächst",
    "bereits",
    "etwa",
    "circa",
    "ca",
    "sogar",
    "sowie",
    "teils",
    "ebenso",
    "ebenfalls",
    // Common verbs (incl. conjugated forms often lowercase in text)
    "ist",
    "sind",
    "war",
    "waren",
    "hat",
    "haben",
    "hatte",
    "hatten",
    "wird",
    "werden",
    "wurde",
    "wurden",
    "worden",
    "bezieht",
    "kann",
    "können",
    "konnte",
    "soll",
    "sollen",
    "sollte",
    "muss",
    "müssen",
    "musste",
    "darf",
    "dürfen",
    "durfte",
    "mag",
    "mögen",
    "möchte",
    "will",
    "wollen",
    "wollte",
    "sein",
    "gewesen",
    "sei",
    "seien",
    "wäre",
    "wären",
    "siehe",
    // Common verb forms that end in -e (1st person singular)
    "schreibe",
    "lerne",
    "mache",
    "habe",
    "gebe",
    "nehme",
    "sehe",
    "komme",
    "finde",
    "denke",
    "sage",
    "frage",
    "gibe",
    "wisse",
    "verstehe",
    "versuche",
    "brauche",
    "suche",
    "arbeite",
    "spiele",
    "lese",
    "höre",
    "glaube",
    // Common past participles
    "fehlgeschlagen",
    // Adjectives
    "gut",
    "groß",
    "klein",
    "alt",
    "neu",
    "lang",
    "kurz",
    "schnell",
    "langsam",
    "viel",
    "wenig",
    "lag",
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
    // prepositions (incl. contracted forms)
    "in",
    "im",
    "ins",
    "an",
    "am",
    "ans",
    "auf",
    "aufs",
    "aus",
    "bei",
    "beim",
    "mit",
    "nach",
    "von",
    "vom",
    "vor",
    "zu",
    "zum",
    "zur",
    "über",
    "übers",
    "unter",
    "unters",
    "durch",
    "durchs",
    "für",
    "fürs",
    "gegen",
    "ohne",
    "um",
    "ums",
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

    /// Does the token immediately to the left mark this position as inside a
    /// noun phrase (determiner / preposition / possessive / spelled-out
    /// number)? This is what separates *"der **Fang**"* (capitalize) from
    /// *"..., **fang** an"* (leave as verb).
    fn is_licensed_by_context(&self, prev: Option<&Token>, document: &Document) -> bool {
        let Some(prev) = prev else {
            return false;
        };
        // Punctuation, numbers, whitespace, symbols to the left never license a
        // noun. Only a genuine word can.
        if !matches!(prev.kind, TokenKind::Word(_)) {
            return false;
        }

        if prev.kind.is_preposition() || prev.kind.is_determiner() {
            return true;
        }

        let prev_chars = document.get_span_content(&prev.span);
        let prev_lower: String = prev_chars
            .iter()
            .map(|c| c.to_lowercase().next().unwrap_or(*c))
            .collect();

        NOUN_PHRASE_LICENSORS.contains(&prev_lower.as_str()) || NUMERALS.contains(&prev_lower)
    }

    /// Decide whether a lowercase, alphabetic, non-sentence-initial word should
    /// be flagged as a miscapitalized noun.
    fn check_if_word_is_noun(
        &self,
        word_chars: &[char],
        prev: Option<&Token>,
        document: &Document,
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

        // Ambiguous noun / verb (or noun / adjective) homograph: only a noun
        // here if the left context licenses a noun phrase.
        self.is_licensed_by_context(prev, document)
    }
}

impl<T: Dictionary> Linter for GermanNounCapitalization<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for paragraph in document.iter_paragraphs() {
            for sentence in paragraph.iter_sentences() {
                let first_word_span = sentence.first_non_whitespace().map(|t| t.span);
                let mut prev: Option<&Token> = None;

                for token in sentence.iter() {
                    if token.kind.is_whitespace() {
                        continue;
                    }

                    if matches!(token.kind, TokenKind::Word(_)) {
                        let word_chars = document.get_span_content(&token.span);

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
                            && self.check_if_word_is_noun(word_chars, prev, document)
                        {
                            let mut replacement: Vec<char> = word_chars.to_vec();
                            if let Some(first_char) = replacement.first_mut() {
                                *first_char =
                                    first_char.to_uppercase().next().unwrap_or(*first_char);
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

                    prev = Some(token);
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
            lints.len() > 0,
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
            lints.len() > 0,
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
            lints.len() >= 1,
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
