//! Adjectives that lost their ending, and nominalized adjectives that took the
//! wrong one.

use crate::{
    Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::grammar::determiners::{
        DeterminerReading, adjective_ending_after, determiner_readings,
    },
    language::german::grammar::noun_gender::NounGender,
    language::german::grammar::noun_phrase::lowercase_of,
    language::german::linting::german_relative_clause_comma::GermanRelativeClauseComma,
    language::german::spell::lexical_classes::NUMERALS,
    linting::{Lint, LintKind, Linter, Suggestion},
    spell::Dictionary,
};

/// Adjectives German does not decline: *eine sexy Betreffzeile*, *ein super
/// Tag*, *ein lila Kleid*, *ein bisschen Zeit*.
const INDECLINABLE: &[&str] = &[
    "sexy",
    "super",
    "klasse",
    "prima",
    "lila",
    "rosa",
    "orange",
    "beige",
    "extra",
    "gratis",
    "okay",
    "cool",
    "wenig",
    "bisschen",
    "paar",
    "viel",
    "mehr",
    "genug",
    "genügend",
    "allerlei",
    "lauter",
    "egal",
    "halb",
    "ganz",
    "eigen",
];

/// Degree words that may stand between the determiner and the adjective: *ein
/// sehr schön Haus* is the same mistake as *ein schön Haus*.
const INTENSIFIERS: &[&str] = &[
    "sehr",
    "ziemlich",
    "recht",
    "ganz",
    "besonders",
    "unglaublich",
    "echt",
    "total",
    "wirklich",
    "äußerst",
    "extrem",
    "so",
    "zu",
];

/// Determiners that are also pronouns: *hat **das** sicher Potenzial*, *ich
/// finde **die** richtig Klasse*, *macht **einem** richtig Lust*. Behind a
/// verb, they are read as the pronoun.
const PRONOMINAL: &[&str] = &["das", "die", "der", "dies", "diese", "einem", "einer"];

/// Forms that are pronouns far more often than determiners: *mit **denen**
/// direkt Kontakt*, *verschwendet **keiner** unnötig Zeit*, *bläst **einem**
/// ganz schön Gegenwind entgegen*. They are not read as determiners here.
const PRONOUNS_FIRST: &[&str] = &[
    "denen", "dessen", "deren", "keiner", "keines", "einer", "einem", "eines",
];

/// Prepositions spelled like an adjective: *eine **laut** Bericht schlecht
/// getarnte Gruppe*.
const ADJECTIVE_PREPOSITIONS: &[&str] = &["laut", "gemäß", "entsprechend", "nahe", "unweit"];

/// Subject pronouns, behind which *meine*, *meinen* are the verb: *wir meinen
/// natürlich Nymphe*.
const SUBJECTS: &[&str] = &["ich", "du", "er", "sie", "es", "wir", "ihr", "man"];

/// Requires the declension ending on an adjective between a determiner and its
/// noun: *der neu Vertrag* → *der neue Vertrag*, *eine lang Reise* → *eine
/// lange Reise*, *sein sehr schön Frau*. And the matching ending on a
/// nominalized adjective: *der Abgeordneter* → *der Abgeordnete*, *ein
/// Angestellte* → *ein Angestellter*.
///
/// An undeclined word in that slot is usually an adverb, which is why the
/// pattern is narrow:
///
/// * only behind a **determiner** — behind a preposition the word is almost
///   always an adverb (*mit wenig Engagement*, *in ganz Ostafrika*, *auf gut
///   Deutsch*);
/// * the word has an **adjective reading** and no preposition reading (*eine
///   laut Polizei überlaute Feier*), and is not one of [`INDECLINABLE`];
/// * the noun **ends the phrase** — punctuation, a verb or the end follows,
///   not another adjective or a determiner. *eine weltweit Beachtung findende
///   Naturkatastrophe* and *die erst Mitte der 1920er Jahre veröffentlichte
///   Untersuchung* are extended attributes, where the adverb is correct;
/// * the noun is not itself a **nominalized adjective**: *ein völlig
///   Fremder*, *einer psychisch Kranken*.
#[derive(Default)]
pub struct GermanAdjectiveForm {
    nouns: NounGender,
}

impl GermanAdjectiveForm {
    pub fn new() -> Self {
        Self::default()
    }

    fn metadata(&self, word: &str) -> Option<std::borrow::Cow<'_, crate::DictWordMetadata>> {
        let chars: Vec<char> = word.chars().collect();
        self.nouns.dictionary().get_word_metadata(&chars)
    }

    fn is_adjective(&self, word: &str) -> bool {
        self.metadata(word).is_some_and(|m| m.is_adjective())
    }

    fn capitalized(token: &Token, document: &Document) -> bool {
        document
            .get_span_content(&token.span)
            .first()
            .is_some_and(|c| c.is_uppercase())
    }

    /// The forms of `stem` the readings allow, as far as the dictionary knows
    /// them.
    fn declined_forms(&self, stem: &str, readings: &[DeterminerReading]) -> Vec<String> {
        let mut forms: Vec<String> = readings
            .iter()
            .filter_map(adjective_ending_after)
            .map(|ending| format!("{stem}{ending}"))
            .filter(|form| self.nouns.dictionary().contains_word_str(form))
            .collect();
        forms.sort();
        forms.dedup();
        forms
    }

    /// *Der neu Vertrag*: an undeclined adjective behind a determiner.
    fn undeclined_adjective(
        &self,
        words: &[&Token],
        at: usize,
        document: &Document,
    ) -> Option<Lint> {
        let determiner = lowercase_of(words[at], document);
        let readings = determiner_readings(&determiner)?;
        if PRONOUNS_FIRST.contains(&determiner.as_str()) {
            return None;
        }
        let previous = at.checked_sub(1).map(|i| words[i]);
        // A relative pronoun behind its comma, or behind a preposition behind
        // one: *…, der allgemein Anklang fand*, *…, in dem ständig Soldaten …*.
        let behind_comma = |i: usize| {
            words
                .get(i)
                .is_some_and(|t| matches!(t.kind, TokenKind::Punctuation(_)))
        };
        if at.checked_sub(1).is_some_and(behind_comma)
            || (previous.is_some_and(|p| p.kind.is_preposition())
                && at.checked_sub(2).is_some_and(behind_comma))
        {
            return None;
        }
        // *wir meinen natürlich Nymphe*: the verb.
        if determiner.starts_with("mein")
            && previous.is_some_and(|p| SUBJECTS.contains(&lowercase_of(p, document).as_str()))
        {
            return None;
        }

        // A pronoun reading behind a verb: *hat das sicher Potenzial*.
        if PRONOMINAL.contains(&determiner.as_str())
            && at
                .checked_sub(1)
                .is_some_and(|i| words[i].kind.is_verb() && !Self::capitalized(words[i], document))
        {
            return None;
        }
        // The numeral or an article after an article: *das eine Wort*.
        if at
            .checked_sub(1)
            .is_some_and(|i| determiner_readings(&lowercase_of(words[i], document)).is_some())
        {
            return None;
        }

        let mut adjective_at = at + 1;
        if words
            .get(adjective_at)
            .is_some_and(|t| INTENSIFIERS.contains(&lowercase_of(t, document).as_str()))
        {
            adjective_at += 1;
        }
        let adjective = words.get(adjective_at)?;
        let noun = words.get(adjective_at + 1)?;
        if !matches!(adjective.kind, TokenKind::Word(_))
            || !matches!(noun.kind, TokenKind::Word(_))
            || Self::capitalized(adjective, document)
            || !Self::capitalized(noun, document)
        {
            return None;
        }

        let word = lowercase_of(adjective, document);
        if INDECLINABLE.contains(&word.as_str())
            || ["e", "en", "er", "es", "em"]
                .iter()
                .any(|ending| word.ends_with(ending))
            || word.ends_with(['a', 'i', 'o', 'y'])
        {
            return None;
        }
        if ADJECTIVE_PREPOSITIONS.contains(&word.as_str())
            || NUMERALS.contains(&word)
            || determiner_readings(&word).is_some()
        {
            return None;
        }
        let metadata = self.metadata(&word)?;
        if !metadata.is_adjective() || metadata.preposition {
            return None;
        }
        // *Das erfordert Können*, *keiner weiß Rat*: a pronoun subject and its
        // verb, when the word can be one.
        let can_be_verb =
            metadata.is_verb() || GermanRelativeClauseComma::infinitive_is_a_verb(&word);
        if can_be_verb && (PRONOMINAL.contains(&determiner.as_str()) || at == 0) {
            return None;
        }

        // The noun has to end the phrase.
        if let Some(after) = words.get(adjective_at + 2)
            && matches!(after.kind, TokenKind::Word(_))
        {
            let after_word = lowercase_of(after, document);
            // An adjective counts only as an extended attribute, with a noun
            // of its own behind it; a participle at the end of the clause is
            // the verb (*sein sehr schön Frau getroffen*).
            let attribute = after.kind.is_adjective()
                && words
                    .get(adjective_at + 3)
                    .is_some_and(|next| Self::capitalized(next, document));
            if determiner_readings(&after_word).is_some()
                || attribute
                || Self::capitalized(after, document)
            {
                return None;
            }
        }
        // *ein völlig Fremder*: the noun is a nominalized adjective.
        let noun_lower = lowercase_of(noun, document);
        if ["e", "en", "er", "es", "em"]
            .iter()
            .any(|ending| noun_lower.ends_with(ending))
            && self.is_adjective(&noun_lower)
        {
            return None;
        }

        let noun_text: String = document.get_span_content(&noun.span).iter().collect();
        let suggestions = match self.nouns.gender_of(&noun_text) {
            Some((gender, _)) => {
                let fitting: Vec<DeterminerReading> = readings
                    .iter()
                    .copied()
                    .filter(|reading| reading.gender == Some(gender))
                    .collect();
                self.declined_forms(&word, &fitting)
            }
            None => Vec::new(),
        };
        Some(Lint {
            span: adjective.span,
            lint_kind: LintKind::Agreement,
            suggestions: suggestions
                .into_iter()
                .map(|form| Suggestion::ReplaceWith(form.chars().collect()))
                .collect(),
            priority: 31,
            message: format!("»{word}« steht zwischen Artikel und Nomen und braucht eine Endung."),
        })
    }

    /// *Der Abgeordneter*, *Ein Angestellte*: a nominalized adjective whose
    /// ending the determiner rules out.
    ///
    /// Only stems that are participles or derived adjectives — see
    /// [`is_derived_adjective_stem`]. A noun like *Junge* (from *jung*) or
    /// *Dichter* (from *dichten*, not *dicht*) is a word of its own. The
    /// recorded gender cannot tell them apart: *Angestellte* carries a stray
    /// feminine, *Dichter* none.
    fn nominalized_adjective(
        &self,
        words: &[&Token],
        at: usize,
        document: &Document,
    ) -> Option<Lint> {
        let determiner = lowercase_of(words[at], document);
        let readings = determiner_readings(&determiner)?;
        let head = words.get(at + 1)?;
        if !matches!(head.kind, TokenKind::Word(_)) || !Self::capitalized(head, document) {
            return None;
        }
        // The head has to end its phrase, with the verb or punctuation behind
        // it: *Der Abgeordneter ging*. Anything else may be an apposition or
        // an extended attribute — *der Abgeordneter Müller*, *Ein Verrückte zu
        // unüberlegten Reaktionen provozierendes Verhalten*.
        if let Some(after) = words.get(at + 2)
            && matches!(after.kind, TokenKind::Word(_))
            && (Self::capitalized(after, document)
                || !GermanRelativeClauseComma::is_verb_token(after, document))
        {
            return None;
        }
        // Behind a comma, the determiner may be a relative pronoun: *Hans
        // Meiser, der Abgeordneter im Bundestag war*.
        if at
            .checked_sub(1)
            .is_some_and(|i| matches!(words[i].kind, TokenKind::Punctuation(_)))
        {
            return None;
        }

        let head_text: String = document.get_span_content(&head.span).iter().collect();
        let lower = head_text.to_lowercase();
        let (stem, ending) = [
            ("er", "er"),
            ("es", "es"),
            ("en", "en"),
            ("em", "em"),
            ("e", "e"),
        ]
        .iter()
        .find_map(|(suffix, ending)| lower.strip_suffix(suffix).map(|stem| (stem, *ending)))?;
        if !is_derived_adjective_stem(stem) || !self.is_adjective(stem) {
            return None;
        }
        // *das Schweigen*: a nominalized infinitive, not an adjective.
        if ending == "en" && self.metadata(&lower).is_some_and(|m| m.is_verb()) {
            return None;
        }

        let allowed: Vec<&str> = readings.iter().filter_map(adjective_ending_after).collect();
        if allowed.is_empty() || allowed.contains(&ending) {
            return None;
        }
        // Only the singular readings have a table; a determiner with plural
        // readings could license another ending.
        if readings.iter().any(|reading| reading.gender.is_none()) && ending == "en" {
            return None;
        }

        let mut suggestions: Vec<String> = allowed
            .iter()
            .map(|wanted| {
                let mut form: String = head_text
                    .chars()
                    .take(head_text.chars().count() - ending.chars().count())
                    .collect();
                form.push_str(wanted);
                form
            })
            .collect();
        suggestions.sort();
        suggestions.dedup();
        Some(Lint {
            span: head.span,
            lint_kind: LintKind::Agreement,
            suggestions: suggestions
                .into_iter()
                .map(|form| Suggestion::ReplaceWith(form.chars().collect()))
                .collect(),
            priority: 31,
            message: format!(
                "»{head_text}« ist ein nominalisiertes Adjektiv. Nach »{determiner}« endet es auf »-{}«.",
                allowed.join("« oder »-")
            ),
        })
    }
}

/// Is `stem` a participle or a derived adjective, the stems German nominalizes
/// freely? A past participle (*abgeordnet*, *angestellt*, *verurteilt*,
/// *gelehrt*), a present participle (*vorsitzend*), or an adjective in
/// *-lich*, *-los*, *-ig* (*jugendlich*, *obdachlos*, *heilig*). A bare
/// adjective like *jung* or *dicht* is not: *der Junge* and *der Dichter* are
/// nouns of their own.
fn is_derived_adjective_stem(stem: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "ge", "be", "ver", "er", "ent", "zer", "an", "ab", "aus", "ein", "vor", "nach", "auf",
        "zu", "über", "unter", "mit", "los", "fest",
    ];
    let participle = stem.ends_with('t') && PREFIXES.iter().any(|prefix| stem.starts_with(prefix));
    participle
        || ["end", "lich", "los", "ig"]
            .iter()
            .any(|suffix| stem.ends_with(suffix))
}

impl Linter for GermanAdjectiveForm {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        for sentence in document.iter_sentences() {
            let words: Vec<&Token> = sentence
                .iter()
                .filter(|token| !token.kind.is_whitespace())
                .collect();
            for at in 0..words.len() {
                if !matches!(words[at].kind, TokenKind::Word(_)) {
                    continue;
                }
                // A capitalized determiner in mid-sentence is part of a name.
                if at > 0 && Self::capitalized(words[at], document) {
                    continue;
                }
                if let Some(lint) = self.undeclined_adjective(&words, at, document) {
                    lints.push(lint);
                } else if let Some(lint) = self.nominalized_adjective(&words, at, document) {
                    lints.push(lint);
                }
            }
        }
        lints
    }

    fn description(&self) -> &str {
        "Prüft die Endung eines Adjektivs zwischen Artikel und Nomen (»der neue Vertrag«) und eines nominalisierten Adjektivs (»der Abgeordnete«)."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanAdjectiveForm;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn reported(text: &str) -> Vec<String> {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanAdjectiveForm::new()
            .lint(&document)
            .into_iter()
            .map(|lint| document.get_span_content_str(&lint.span))
            .collect()
    }

    #[test]
    fn an_undeclined_adjective_is_reported() {
        assert_eq!(reported("Der neu Vertrag wurde unterschrieben."), ["neu"]);
        assert_eq!(reported("Es ist eine lang Reise."), ["lang"]);
        assert_eq!(reported("Mein klein Haus ist zu klein."), ["klein"]);
        assert_eq!(reported("Wir haben einen gemeinsam Vater."), ["gemeinsam"]);
        assert_eq!(
            reported("Ich habe gestern seine sehr schön Frau getroffen."),
            ["schön"]
        );
        assert_eq!(reported("Das ist keine gut Überprüfung."), ["gut"]);
    }

    #[test]
    fn a_declined_or_indeclinable_adjective_is_quiet() {
        for text in [
            "Der neue Vertrag wurde unterschrieben.",
            "Mein kleines Haus ist zu klein.",
            "Ich habe einen sexy Haarschnitt.",
            "Es wird immer ein wenig Material entfernt.",
            "Er ist ein völlig Fremder für mich.",
            "Wenn sie es schaffen, hat das sicher Potenzial.",
            "Ich finde die richtig Klasse!",
            "Das kann einem ganz schön Angst machen.",
            "Eine weltweit Beachtung findende Naturkatastrophe.",
            "Die erst Mitte der 1920er Jahre veröffentlichte Untersuchung.",
            "Aufgrund einer laut Polizei überlauten Feier haben sich Nachbarn beschwert.",
            "Das eine bedeutet Gefahr und das andere Gelegenheit.",
            "Er betrieb dies jedoch mit wenig Engagement.",
            "Armstrong gehörte zu den neun Testpiloten.",
            "Eventuell muss man mit denen direkt Kontakt aufnehmen.",
            "1879 lag der Entwurf vor, der allgemein Anklang fand.",
            "Das erfordert Können und Hingabe.",
            "Alle jammern, aber keiner weiß Rat.",
            "Wir meinen natürlich Nymphe.",
            "Ist das ein Ferrari oder ein Lamborghini?",
            "Hoffentlich verschwendet keiner unnötig Zeit.",
        ] {
            assert!(reported(text).is_empty(), "{text}: {:?}", reported(text));
        }
    }

    #[test]
    fn a_nominalized_adjective_takes_the_determiners_ending() {
        assert_eq!(
            reported("Der Abgeordneter ging nach Hause."),
            ["Abgeordneter"]
        );
        assert_eq!(reported("Der Angestellter streikt."), ["Angestellter"]);
        assert_eq!(reported("Ein Angestellte streikt."), ["Angestellte"]);
    }

    #[test]
    fn a_correct_nominalized_adjective_is_quiet() {
        for text in [
            "Der Abgeordnete ging nach Hause.",
            "Ein Abgeordneter ging ans Rednerpult.",
            "Ein Junge ging spazieren.",
            "Der Dichter schrieb ein Gedicht.",
            "Hans Meiser, der Abgeordneter im Bundestag war, hatte niemals Feierabend.",
            "Ein Abgeordnete betreffendes Problem ist die Work-Life-Balance.",
            "Der Lehrer kommt.",
            "Das Schweigen von gestern rechtfertigt nichts.",
            "Ein Verrückte zu unüberlegten Reaktionen provozierendes Verhalten.",
        ] {
            assert!(reported(text).is_empty(), "{text}: {:?}", reported(text));
        }
    }
}
