//! *Ich möchte dich zu besuchen*: a modal takes the bare infinitive.

use crate::{
    Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::{
        grammar::{
            determiners::determiner_readings,
            noun_phrase::lowercase_of,
            subjects::{irregular_finite_verb_lemma, subject_pronoun},
            verbs::{
                INFINITIVE_GROUP_OPENERS, looks_like_participle, without_zu_infix, zu_infix_parts,
            },
        },
        linting::german_relative_clause_comma::GermanRelativeClauseComma,
        spell::curated_german_dictionary,
    },
    linting::{Lint, LintKind, Linter, Suggestion},
    spell::Dictionary,
};

/// The verbs that govern a bare infinitive, by infinitive: the six modals, and
/// *werden* for the future and *würde*.
///
/// *wissen* is in the same irregular table but takes *zu* (*er weiß sich zu
/// helfen*), and so do *brauchen*, *haben* and *sein* (*Sie haben zu gehen*).
const BARE_INFINITIVE_GOVERNORS: &[&str] = &[
    "können", "müssen", "wollen", "sollen", "dürfen", "mögen", "werden",
];

/// The indefinite objects of a *zu*-infinitive that a modal can take on their
/// own, without an infinitive of its own: *Ich möchte etwas zu trinken*, *Er
/// will nichts zu essen*. Here *möchte* is the full verb "want".
const INDEFINITE_OBJECTS: &[&str] = &["etwas", "nichts", "was", "viel", "wenig", "genug", "mehr"];

/// Reports the *zu* in front of an infinitive that a modal governs: *Ich
/// möchte dich zu besuchen* → *Ich möchte dich besuchen*, *weil ich dich zu
/// sehen möchte* → *weil ich dich sehen möchte*, *Ich möchte Leute
/// kennenzulernen* → *kennenlernen*.
///
/// Learners carry the *zu* over from the verbs that do take it (*versuchen*,
/// *beginnen*, *hoffen*). The modal and the infinitive have to stand in the
/// same clause with the infinitive at its end — main clause — or the
/// infinitive directly before a modal that ends it — subordinate clause. No
/// other verb may stand between modal and *zu*: *Ich muss versuchen zu
/// schlafen* is correct, because the *zu* belongs to *versuchen*.
#[derive(Default)]
pub struct GermanModalZuInfinitive;

impl GermanModalZuInfinitive {
    fn governs_bare_infinitive(word: &str) -> bool {
        irregular_finite_verb_lemma(word)
            .is_some_and(|lemma| BARE_INFINITIVE_GOVERNORS.contains(&lemma))
    }

    fn is_lowercase(token: &Token, document: &Document) -> bool {
        document
            .get_span_content(&token.span)
            .first()
            .is_some_and(|c| c.is_lowercase())
    }

    /// A lower-case word that can be the infinitive behind *zu*.
    fn is_infinitive(token: &Token, document: &Document) -> bool {
        let word = lowercase_of(token, document);
        Self::is_lowercase(token, document)
            && word.ends_with('n')
            && (word == "sein"
                || GermanRelativeClauseComma::is_verb_token(token, document)
                || GermanRelativeClauseComma::infinitive_is_a_verb(&word))
    }

    /// An infinitive with its *zu* inside, *anzurufen*, and the infinitive
    /// without it, which the dictionary has to know as a verb.
    ///
    /// Not when prefix and *zu* make a word of their own: *hinzukommen* is
    /// *hinzu* + *kommen*, not *hin* + *zu* + *kommen*.
    fn zu_infix_infinitive(token: &Token, document: &Document) -> Option<String> {
        if !Self::is_lowercase(token, document) {
            return None;
        }
        let (prefix, rest) = zu_infix_parts(&lowercase_of(token, document))?;
        let dictionary = curated_german_dictionary();
        let is_word = |word: &str| {
            let chars: Vec<char> = word.chars().collect();
            dictionary.get_word_metadata(&chars)
        };
        if is_word(&format!("{prefix}zu")).is_some() {
            return None;
        }
        let bare = format!("{prefix}{rest}");
        is_word(&bare)
            .is_some_and(|metadata| metadata.is_verb())
            .then_some(bare)
    }

    /// *um*, *ohne*, *statt*, and *als* in *lieber laufen als zu warten*: the
    /// *zu* behind them belongs to their own group.
    fn opens_group(word: &str) -> bool {
        word == "als" || INFINITIVE_GROUP_OPENERS.contains(&word)
    }

    /// A verb between the modal and the *zu*, which then governs the *zu*
    /// itself: *versuchen*, *bereit sein*, *aufgefordert*. Only an infinitive
    /// or a participle can stand there, so only a word ending in *n* or *t*
    /// counts — not the particle *bitte*, which is also *ich bitte* — and not a
    /// determiner with a verb reading (*einen*, *sein Auto*).
    fn is_intervening_verb(clause: &[&Token], at: usize, document: &Document) -> bool {
        let token = clause[at];
        let word = lowercase_of(token, document);
        if !Self::is_lowercase(token, document) || !(word.ends_with('n') || word.ends_with('t')) {
            return false;
        }
        if word == "sein" {
            // The infinitive, unless a noun follows: *sein Auto*.
            return !clause
                .get(at + 1)
                .is_some_and(|next| !Self::is_lowercase(next, document));
        }
        // A participle, which the dictionary often files as an adjective:
        // *Er wurde aufgefordert zu kommen*, *Er wird verpflichtet zu zahlen*.
        // Any other adjective with a stray verb reading is no verb here:
        // *Ich will unbedingt neue Leute kennenlernen*.
        let participle = looks_like_participle(&word)
            || ["be", "ver", "er", "ent", "zer", "emp"]
                .iter()
                .any(|prefix| word.starts_with(prefix));
        if token.kind.is_adjective() {
            return participle;
        }
        GermanRelativeClauseComma::is_verb_token(token, document)
            && !token.kind.is_determiner()
            && !token.kind.is_pronoun()
            && determiner_readings(&word).is_none()
    }

    fn report(clause: &[&Token], zu: usize, modal: &str, document: &Document) -> Lint {
        let token = clause[zu];
        let word = lowercase_of(token, document);
        if word != "zu" {
            // *kennenzulernen* → *kennenlernen*.
            let fixed = without_zu_infix(&word).unwrap_or_else(|| word.clone());
            return Lint {
                span: token.span,
                lint_kind: LintKind::Grammar,
                suggestions: vec![Suggestion::replace_with_match_case(
                    fixed.chars().collect(),
                    document.get_span_content(&token.span),
                )],
                message: format!(
                    "Nach »{modal}« steht der Infinitiv ohne »zu«: »{fixed}«, nicht »{word}«."
                ),
                priority: 31,
            };
        }
        let infinitive = lowercase_of(clause[zu + 1], document);
        let mut span = token.span;
        span.end = clause[zu + 1].span.start;
        // *Wir müssen einen Zahn zu legen*: the particle of *zulegen*, written
        // apart. Then joining is the other fix, offered second.
        let joined = format!("zu{infinitive}");
        let joined_chars: Vec<char> = joined.chars().collect();
        let is_verb = curated_german_dictionary()
            .get_word_metadata(&joined_chars)
            .is_some_and(|metadata| metadata.is_verb());
        let mut suggestions = vec![Suggestion::Remove];
        let mut message =
            format!("Nach »{modal}« steht der Infinitiv ohne »zu«: »{modal} … {infinitive}«.");
        if is_verb {
            suggestions.push(Suggestion::ReplaceWith(vec!['z', 'u']));
            message.push_str(&format!(" Oder ist das Verb »{joined}« gemeint?"));
        }
        Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions,
            message,
            priority: 31,
        }
    }

    /// *werden* is also the copula, and then the *zu* belongs to the
    /// predicate: *Danach wird es leichter, eine Wohnung zu finden*, *Die
    /// Arbeit würde eine Möglichkeit sein, … zu verbessern*. The future and
    /// *würde* are only read with a personal subject next to them: *Ich würde
    /// gern dich zu sehen*, *dass ich dich anzurufen werde*.
    fn has_personal_subject(
        lower: &[String],
        governor: usize,
        range: std::ops::Range<usize>,
    ) -> bool {
        if irregular_finite_verb_lemma(&lower[governor]) != Some("werden") {
            return true;
        }
        range
            .filter(|&at| at < lower.len())
            .any(|at| subject_pronoun(&lower[at]).is_some())
    }

    fn lint_clause(clause: &[&Token], document: &Document, lints: &mut Vec<Lint>) {
        let lower: Vec<String> = clause
            .iter()
            .map(|token| lowercase_of(token, document))
            .collect();
        let Some(last) = clause.len().checked_sub(1) else {
            return;
        };
        let is_zu_infix = |at: usize| Self::zu_infix_infinitive(clause[at], document).is_some();
        let indefinite_before =
            |zu: usize| zu > 0 && INDEFINITE_OBJECTS.contains(&lower[zu - 1].as_str());

        // Main clause: the modal somewhere in front, the infinitive last.
        let zu = if is_zu_infix(last) {
            Some(last)
        } else if last >= 1
            && lower[last - 1] == "zu"
            && Self::is_infinitive(clause[last], document)
        {
            Some(last - 1)
        } else {
            None
        };
        if let Some(zu) = zu
            && !indefinite_before(zu)
            && let Some(modal) = (0..zu)
                .rev()
                .find(|&at| Self::governs_bare_infinitive(&lower[at]))
            // An infinitive that looks like the finite modal: *anpassen zu
            // können und … zu reduzieren*, *nie ruhig werden zu können*.
            && !(modal > 0 && lower[modal - 1] == "zu")
            && modal + 1 < zu
            && Self::has_personal_subject(&lower, modal, modal.saturating_sub(1)..modal + 2)
            && !(modal + 1..zu).any(|at| {
                Self::opens_group(&lower[at]) || Self::is_intervening_verb(clause, at, document)
            })
        {
            lints.push(Self::report(clause, zu, &lower[modal], document));
            return;
        }

        // Subordinate clause: *zu* and the infinitive directly before a modal
        // that ends the clause.
        if last >= 1 && Self::governs_bare_infinitive(&lower[last]) {
            let zu = if is_zu_infix(last - 1) {
                Some(last - 1)
            } else if last >= 2
                && lower[last - 2] == "zu"
                && Self::is_infinitive(clause[last - 1], document)
            {
                Some(last - 2)
            } else {
                None
            };
            if let Some(zu) = zu
                && !indefinite_before(zu)
                && Self::has_personal_subject(&lower, last, 0..zu)
                && !(0..zu).any(|at| Self::opens_group(&lower[at]))
            {
                lints.push(Self::report(clause, zu, &lower[last], document));
            }
        }
    }
}

impl Linter for GermanModalZuInfinitive {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let words: Vec<&Token> = sentence
                .iter()
                .filter(|token| !token.kind.is_whitespace())
                .collect();
            for clause in words.split(|token| matches!(token.kind, TokenKind::Punctuation(_))) {
                Self::lint_clause(clause, document, &mut lints);
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Nach einem Modalverb steht der Infinitiv ohne »zu«: »Ich möchte dich besuchen«, nicht »Ich möchte dich zu besuchen«."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanModalZuInfinitive;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn fixed(text: &str) -> Vec<String> {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        let mut lints = GermanModalZuInfinitive.lint(&document);
        lints.sort_by_key(|lint| std::cmp::Reverse(lint.span.start));
        let mut chars: Vec<char> = text.chars().collect();
        for lint in &lints {
            if let Some(suggestion) = lint.suggestions.first() {
                suggestion.apply(lint.span, &mut chars);
            }
        }
        if lints.is_empty() {
            Vec::new()
        } else {
            vec![chars.into_iter().collect()]
        }
    }

    #[test]
    fn zu_after_a_modal_is_reported() {
        for (wrong, right) in [
            ("Ich möchte dich zu besuchen.", "Ich möchte dich besuchen."),
            (
                "Er wollte zuerst eine Katze zu haben.",
                "Er wollte zuerst eine Katze haben.",
            ),
            (
                "Ich würde gern dich zu sehen.",
                "Ich würde gern dich sehen.",
            ),
            (
                "Kannst du mir bitte zu helfen?",
                "Kannst du mir bitte helfen?",
            ),
            (
                "Wir müssen heute die Wohnung zu putzen.",
                "Wir müssen heute die Wohnung putzen.",
            ),
            (
                "Ich möchte gern bei dir zu sein.",
                "Ich möchte gern bei dir sein.",
            ),
            (
                "Er möchte sein Haus anzustreichen.",
                "Er möchte sein Haus anstreichen.",
            ),
            (
                "Ich möchte neue Leute kennenzulernen.",
                "Ich möchte neue Leute kennenlernen.",
            ),
            (
                "Ich will unbedingt neue Leute kennenzulernen.",
                "Ich will unbedingt neue Leute kennenlernen.",
            ),
            (
                "Ich weiß, dass ich dich zu besuchen muss.",
                "Ich weiß, dass ich dich besuchen muss.",
            ),
            (
                "Man kann auch in der Liste nachzuschauen.",
                "Man kann auch in der Liste nachschauen.",
            ),
            (
                "Ich werde dich morgen anzurufen.",
                "Ich werde dich morgen anrufen.",
            ),
        ] {
            assert_eq!(fixed(wrong), [right], "{wrong}");
        }
    }

    /// The particle of *zulegen* written apart: removing *zu* and joining
    /// it are both offered.
    #[test]
    fn a_separated_particle_is_offered_both_fixes() {
        let dict = combined_german_dictionary();
        let document = Document::new("Wir müssen einen Zahn zu legen.", &PlainGerman, &dict);
        let lints = GermanModalZuInfinitive.lint(&document);
        assert_eq!(lints.len(), 1);
        assert_eq!(lints[0].suggestions.len(), 2);
    }

    #[test]
    fn a_zu_that_belongs_elsewhere_is_quiet() {
        for text in [
            "Ich möchte dich besuchen.",
            "Ich möchte etwas zu trinken.",
            "Er will nichts zu essen.",
            "Er hat nichts zu tun.",
            "Ich muss versuchen zu schlafen.",
            "Ich möchte gern zu Hause bleiben.",
            "Er weiß sich zu helfen.",
            "Wir müssen bereit sein zu helfen.",
            "Du musst lernen, Geduld zu haben.",
            "Man muss arbeiten, um Geld zu verdienen.",
            "Ich möchte lieber laufen als zu warten.",
            "Er kann zu schnell laufen.",
            "Ich will dir nicht zu nahe treten.",
            "Ich will zu dir.",
            "Die Aufgabe wird nicht leicht zu lösen sein.",
            "Ich möchte, dass du versuchst zu kommen.",
            "Sie haben zu gehen.",
            "Ich möchte Leute kennenlernen.",
            "Er kann beginnen, sich fremd zu fühlen.",
            "Er wurde aufgefordert zu kommen.",
            "Er wird verpflichtet zu zahlen.",
            "Linux könnte später hinzukommen.",
            "Ziel ist es, sich anpassen zu können und Kosten zu reduzieren.",
            "Er scheint nie ruhig werden zu können.",
            "Danach wird es leichter eine Wohnung zu mieten.",
            "Dazu würde es besser sein in einem Laden zu arbeiten.",
            "Die Arbeit würde eine Möglichkeit meine Sprache zu verbessern.",
        ] {
            assert!(fixed(text).is_empty(), "{text}");
        }
    }
}
