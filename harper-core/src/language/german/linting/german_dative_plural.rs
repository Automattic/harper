//! *mit den Kinder*, *seit drei Jahre*: the dative plural ends in *-n*.

use crate::{
    Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::grammar::{
        determiners::{determiner_readings, is_plural_only_quantifier},
        noun_gender::NounGender,
        noun_phrase::{self, closed_head_after, closes_the_phrase},
        prepositions::preposition_government,
        verbs::looks_like_participle,
    },
    language::german::spell::lexical_classes::NUMERALS,
    language::morphology::{Case, CaseSet, Number},
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Reports a noun plural without the *-n* the dative plural takes: *mit den
/// Kinder* → *Kindern*, *mit meinen Freunde* → *Freunden*, *seit drei Jahre*
/// → *Jahren*, *den Kinder gegeben* → *Kindern*. The ending is the one German
/// noun inflection without exceptions, apart from plurals in *-s* and the
/// Latin and Greek ones.
///
/// Two places force the dative plural:
///
/// * a **determiner whose every plural reading is dative** — *den*, *diesen*,
///   *meinen*, *keinen*, *allen*, *vielen* — in front of a plural noun. *den
///   Lehrer* is the accusative singular, so the noun has to be known as a
///   plural: `NounGender::lacks_dative_plural_n` reads the marker hunspell's
///   plural affixes put on *Kinder*, *Freunde*, *Bücher*.
/// * a **preposition that only governs the dative** — *mit*, *seit*, *bei*,
///   *von*, *aus*, *nach*, *zu* — with nothing but numbers and adjectives
///   before the plural: *seit drei Jahre*, *mit kleine Kinder*.
///
/// Behind a preposition that does not take the dative — *für den Kinder* —
/// the determiner is what is wrong, and `GermanPrepositionCase` says so.
pub struct GermanDativePlural {
    nouns: NounGender,
}

impl Default for GermanDativePlural {
    fn default() -> Self {
        Self::new()
    }
}

impl GermanDativePlural {
    pub fn new() -> Self {
        Self {
            nouns: NounGender::new(),
        }
    }

    fn lower(token: &Token, document: &Document) -> String {
        noun_phrase::lowercase_of(token, document)
    }

    fn capitalized(token: &Token, document: &Document) -> bool {
        document
            .get_span_content(&token.span)
            .first()
            .is_some_and(|c| c.is_uppercase())
    }

    /// Does the determiner mean the dative whenever its noun is a plural?
    fn plural_is_dative(word: &str) -> bool {
        determiner_readings(word).is_some_and(|readings| {
            let plural: Vec<_> = readings
                .iter()
                .filter(|reading| reading.number() == Number::Plural)
                .collect();
            !plural.is_empty() && plural.iter().all(|reading| reading.case == Case::Dative)
        })
    }

    /// `Some(conjunction)` for a preposition that only governs the dative;
    /// `conjunction` says whether it also opens a clause (*seit*), whose
    /// subject is a nominative: *seit drei Jahre vergangen sind*.
    ///
    /// *seit* is missing from the preposition table, because the conjunction
    /// takes a nominative subject behind its determiner (*seit das Kind hier
    /// ist*). Here no determiner follows it, and it is read as the
    /// conjunction-preposition it is: *seit drei Jahre* only when the phrase
    /// does not end in a verb.
    ///
    /// *zu*, *nach* and *aus* are left out: *zu* is the degree word and the
    /// infinitive marker as often (*viel zu komplexen*, *bis zu sieben*, *die
    /// zu erwartenden*) and keeps the old dative singular (*zu Tage*), and
    /// *nach* and *aus* close a phrase as postpositions (*meiner Meinung
    /// nach*, *von dort aus*).
    fn governs_only_the_dative(word: &str) -> Option<bool> {
        if word == "seit" {
            return Some(true);
        }
        if matches!(word, "zu" | "nach" | "aus") {
            return None;
        }
        preposition_government(word)
            .filter(|preposition| preposition.cases == CaseSet::from(Case::Dative))
            .map(|preposition| preposition.also_a_conjunction)
    }

    /// A preposition right in front that does not take the dative: then the
    /// determiner is the mistake, not the noun.
    fn after_a_non_dative_preposition(words: &[&Token], at: usize, document: &Document) -> bool {
        at.checked_sub(1)
            .and_then(|previous| preposition_government(&Self::lower(words[previous], document)))
            .is_some_and(|preposition| !preposition.cases.contains(CaseSet::from(Case::Dative)))
    }

    /// The head of a phrase without a determiner that starts at `start`,
    /// behind a dative-only preposition or at a quantifier: numbers and
    /// adjectives may stand in front of it, and the phrase has to end behind
    /// the noun.
    fn bare_head_from(
        words: &[&Token],
        start: usize,
        document: &Document,
        verb_closes: bool,
    ) -> Option<usize> {
        let mut at = start;
        let mut counted = false;
        while let Some(token) = words.get(at) {
            let word = Self::lower(token, document);
            let number = (NUMERALS.contains(&word) && !word.starts_with("ein"))
                // *mit vielen Leute*, *bei beiden Kinder*: the plural
                // quantifiers in their dative form.
                || word
                    .strip_suffix('n')
                    .is_some_and(is_plural_only_quantifier);
            let adjective = !Self::capitalized(token, document)
                && token.kind.is_adjective()
                && !token.kind.is_determiner()
                && determiner_readings(&word).is_none();
            if !(number || adjective) {
                break;
            }
            counted |= number;
            at += 1;
        }
        // Only a spelled-out number or a quantifier makes the phrase a plural
        // for sure. An adjective alone does not (*zu fairen Beiträge* is real,
        // *viel zu komplexen Befehlssätze* is not), and digits are mostly
        // years: *seit 1998 Feldversuche*.
        if !counted {
            return None;
        }
        let head = words.get(at)?;
        if !Self::capitalized(head, document)
            || determiner_readings(&Self::lower(head, document)).is_some()
        {
            return None;
        }
        let content = document.get_full_content();
        if content.get(head.span.end) == Some(&'-')
            || (head.span.start > 0 && content.get(head.span.start - 1) == Some(&'-'))
        {
            return None;
        }
        let ends = match words.get(at + 1) {
            None => true,
            Some(next) if !matches!(next.kind, TokenKind::Word(_)) => true,
            Some(next) => {
                !Self::capitalized(next, document)
                    && (closes_the_phrase(&Self::lower(next, document))
                        || (verb_closes
                            && next.kind.is_verb()
                            && (!next.kind.is_adjective()
                                || looks_like_participle(&Self::lower(next, document)))))
            }
        };
        ends.then_some(at)
    }

    fn report(&self, head: &Token, document: &Document) -> Option<Lint> {
        let text = document.get_span_content_str(&head.span);
        if !self.nouns.lacks_dative_plural_n(&text) {
            return None;
        }
        let fixed = format!("{text}n");
        Some(Lint {
            span: head.span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::ReplaceWith(fixed.chars().collect())],
            message: format!("Im Dativ Plural endet das Nomen auf -n: »{fixed}«, nicht »{text}«."),
            priority: 30,
        })
    }
}

impl Linter for GermanDativePlural {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let words: Vec<&Token> = sentence
                .iter()
                .filter(|token| !token.kind.is_whitespace())
                .collect();
            let phrases = noun_phrase::phrases(&words, document);
            let mut reported = Vec::new();

            for at in 0..words.len() {
                let token = words[at];
                if !matches!(token.kind, TokenKind::Word(_)) {
                    continue;
                }
                // A capital in mid-sentence is a name: *Den Haag*.
                if at > 0 && Self::capitalized(token, document) {
                    continue;
                }
                let word = Self::lower(token, document);

                // *in vielen Länder*: a plural quantifier in *-n* is the
                // dative whatever governs it — the accusative is *viele*.
                let quantifier = word
                    .strip_suffix('n')
                    .is_some_and(is_plural_only_quantifier);
                // Only straight behind a preposition that can take the
                // dative: behind an article the quantifier is weakly declined
                // in any case — *die beiden Länder*, *eines der vielen
                // Programme* — and at the start it is *Vielen Grüße*.
                let after_a_dative_preposition = at
                    .checked_sub(1)
                    .and_then(|previous| {
                        preposition_government(&Self::lower(words[previous], document))
                    })
                    .is_some_and(|preposition| {
                        preposition.cases.contains(CaseSet::from(Case::Dative))
                    });
                let head = if quantifier && after_a_dative_preposition {
                    Self::bare_head_from(&words, at, document, true)
                } else if Self::plural_is_dative(&word) {
                    if Self::after_a_non_dative_preposition(&words, at, document) {
                        continue;
                    }
                    closed_head_after(document, &words, &phrases, at, true)
                } else if let Some(conjunction) = Self::governs_only_the_dative(&word)
                    && words.get(at + 1).is_some_and(|next| {
                        determiner_readings(&Self::lower(next, document)).is_none()
                    })
                {
                    Self::bare_head_from(&words, at + 1, document, !conjunction)
                } else {
                    None
                };

                if let Some(head) = head
                    && !reported.contains(&head)
                    && let Some(lint) = self.report(words[head], document)
                {
                    reported.push(head);
                    lints.push(lint);
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Verlangt das -n im Dativ Plural: »mit den Kindern«, »seit drei Jahren«."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanDativePlural;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn fixes(text: &str) -> Vec<String> {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanDativePlural::new()
            .lint(&document)
            .into_iter()
            .flat_map(|lint| lint.suggestions)
            .map(|suggestion| suggestion.to_string())
            .collect()
    }

    #[test]
    fn a_dative_plural_without_n_is_reported() {
        for (text, fixed) in [
            ("Ich spiele mit den Kinder.", "Kindern"),
            ("Ich gehe mit meinen Freunde ins Kino.", "Freunden"),
            ("Ich wohne seit drei Jahre in Berlin.", "Jahren"),
            ("Ich habe es den Kinder gegeben.", "Kindern"),
            ("Ich spreche mit vielen Leute.", "Leuten"),
            ("Von zwei Freunde habe ich das gehört.", "Freunden"),
            ("In vielen Länder ist das so.", "Ländern"),
            ("In allen Länder ist das so.", "Ländern"),
            ("Er liest gern in seinen Bücher.", "Büchern"),
            ("Sie spricht mit beiden Brüder.", "Brüdern"),
        ] {
            assert_eq!(fixes(text), [format!("Replace with: “{fixed}”")], "{text}");
        }
    }

    #[test]
    fn a_correct_or_other_case_is_quiet() {
        for text in [
            "Ich spiele mit den Kindern.",
            "Ich wohne seit drei Jahren in Berlin.",
            "Ich sehe den Lehrer.",
            "Ich sehe den Räuber.",
            "Die Kinder spielen.",
            "Ich kaufe Bücher für die Kinder.",
            "Er fährt mit den Autos.",
            "Wir sprechen mit den Lehrern.",
            "Ich habe drei Kinder.",
            "Für den Kinder ist das gut.",
            "Mit den Leuten spricht er gern.",
            "Ich denke an die Kinder.",
            "Er gibt den Kindern Bücher.",
            "Seit Jahren wohnt er hier.",
            "Er kam mit der Frau.",
            "Vielen Dank für alles.",
            "Die beiden Länder arbeiten zusammen.",
            "Er nutzt eines der beiden Programme.",
            "Ich denke an die vielen Kinder.",
            "Er hat mit vielen Menschen gesprochen.",
            "Seit drei Jahre vergangen sind, ist alles anders.",
            "Die Befehle sind viel zu komplexen Befehlssätze.",
            "Er bietet seit 1998 Feldversuche an.",
            "Bis zu sieben Geräte sind erlaubt.",
            "Der Fehler dürfte nur selten zu Tage treten.",
            "Das verdient meiner Meinung nach fünf Sterne.",
        ] {
            assert!(fixes(text).is_empty(), "{text}: {:?}", fixes(text));
        }
    }
}
