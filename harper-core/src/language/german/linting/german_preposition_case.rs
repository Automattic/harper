//! Checks the case of a determiner directly after a preposition.

use crate::{
    Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::grammar::determiners::{
        determiner_cases, determiner_readings, forms_in_case,
    },
    language::german::grammar::prepositions::preposition_government,
    language::morphology::CaseSet,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Determiners that stand in a fixed expression and are correct there whatever
/// the preposition in front of them: *trotz allem*, *trotz alledem*, *von alles
/// entscheidender Bedeutung*.
const FIXED_PHRASE_DETERMINERS: &[&str] = &["allem", "alledem", "alles"];

/// Catches a determiner in the wrong case after a preposition: *"**wegen dem**
/// Wetter"* needs the genitive, *"**für dem** Kind"* the accusative.
///
/// The check is the set intersection LanguageTool performs with `retainAll`:
/// the preposition names the cases it governs, the determiner carries every
/// case it can be read in, and an empty intersection is the error. Both sides
/// come from [`crate::language::german::grammar`], so nothing here depends on
/// the 109,000 noun entries — of which 99.6% carry no gender and none carries a
/// case.
///
/// That is also the limit of the rule. It sees the determiner alone, so it
/// cannot catch a mistake that the determiner survives: *mit den Freund* is
/// wrong, but *den* is a perfectly good dative plural and only the singular
/// *Freund* gives it away. Catching those needs gender on the nouns.
///
/// Four things look like the pattern and are not, and each is turned away:
///
/// * *"während **das** Kind schlief"* — a subordinate clause, whose subject is
///   nominative. Reported only in the dative for such words.
/// * *"in **des** Kaisers Namen"* — a genitive standing in front of its noun,
///   which is elevated but correct after any preposition.
/// * *"um **der** Sache **willen**"* — a circumposition, where the case belongs
///   to *willen* and not to *um*.
/// * *"von **Der** Spiegel"* — a capitalized determiner is part of a name.
pub struct GermanPrepositionCase;

impl GermanPrepositionCase {
    /// German names of a set of cases, joined for the message. `article` is
    /// prefixed to each: *verlangt **den** Genitiv*, but *steht **im** Dativ*.
    fn label(cases: CaseSet, article: &str) -> String {
        let names = [
            (CaseSet::NOMINATIVE, "Nominativ"),
            (CaseSet::ACCUSATIVE, "Akkusativ"),
            (CaseSet::DATIVE, "Dativ"),
            (CaseSet::GENITIVE, "Genitiv"),
        ];

        let present: Vec<String> = names
            .iter()
            .filter(|(flag, _)| cases.contains(*flag))
            .map(|(_, name)| format!("{article} {name}"))
            .collect();

        match present.split_last() {
            Some((last, [])) => last.clone(),
            Some((last, rest)) => format!("{} oder {last}", rest.join(", ")),
            None => String::new(),
        }
    }
}

impl Linter for GermanPrepositionCase {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let tokens: Vec<&Token> = sentence.iter().collect();

            let word_at = |index: usize| -> Option<String> {
                tokens
                    .get(index)
                    .filter(|token| matches!(token.kind, TokenKind::Word(_)))
                    .map(|token| document.get_span_content(&token.span).iter().collect())
            };
            let first_word = tokens
                .iter()
                .position(|token| matches!(token.kind, TokenKind::Word(_)));

            for index in 0..tokens.len() {
                let (Some(preposition_text), Some(determiner_text)) =
                    (word_at(index), word_at(index + 2))
                else {
                    continue;
                };
                if !tokens[index + 1].kind.is_whitespace() {
                    continue;
                }

                let Some(government) = preposition_government(&preposition_text) else {
                    continue;
                };

                // `MIT` the institute, `Bei` the warlord, `Bar` the room. A
                // German preposition is lower case unless it opens the
                // sentence.
                if is_capitalized(&preposition_text) && first_word != Some(index) {
                    continue;
                }

                // *von Die Zeit*: a capitalized determiner belongs to a name.
                // It can never be sentence-initial here, because a preposition
                // precedes it.
                if is_capitalized(&determiner_text) {
                    continue;
                }

                let Some(cases) = determiner_cases(&determiner_text) else {
                    continue;
                };

                if cases.intersects(government.cases) {
                    continue;
                }

                // A noun phrase needs a noun. Without a following word the
                // determiner is a pronoun or a fragment, and the rule has
                // nothing to say about it.
                let Some(next_word) = word_at(index + 4) else {
                    continue;
                };

                // *ein und derselbe*, *ein oder mehrere*, *ein bis zwei*: the
                // determiner is part of a fixed coordination and stays
                // uninflected.
                if ["und", "oder", "bis"].contains(&next_word.as_str()) {
                    continue;
                }

                if FIXED_PHRASE_DETERMINERS.contains(&determiner_text.as_str()) {
                    continue;
                }

                // *während das Kind schlief*: a clause, not a genitive that has
                // gone wrong. Only a dative rules the clause reading out.
                if government.also_a_conjunction && !cases.contains(CaseSet::DATIVE) {
                    continue;
                }

                // *in des Kaisers Namen*: a genitive in front of its noun is
                // correct after any preposition.
                if cases == CaseSet::GENITIVE && !government.cases.contains(CaseSet::GENITIVE) {
                    continue;
                }

                let rest = index + 3..tokens.len();

                // *um dem Leser eine spannende Handlung zu bieten*: the case
                // belongs to the infinitive, not to *um*.
                if government.also_an_infinitive_clause
                    && rest
                        .clone()
                        .filter_map(word_at)
                        .any(|word| word == "zu" || is_infinitive_with_zu(&word))
                {
                    continue;
                }

                // *um der Sache willen*: the case belongs to the second half
                // of the circumposition.
                if rest
                    .clone()
                    .any(|at| word_at(at).is_some_and(|word| word.eq_ignore_ascii_case("willen")))
                {
                    continue;
                }

                // *seiner Ansicht nach*, *ihm zufolge*, *von wegen*: the word
                // follows its noun phrase, and what comes after it starts a new
                // one.
                if government.also_a_postposition {
                    let previous = index.checked_sub(2).and_then(word_at);
                    if previous.is_some_and(|previous| {
                        closes_a_noun_phrase(&previous, first_word == index.checked_sub(2))
                    }) {
                        continue;
                    }

                    // *von wo aus*, *von dort aus*: the frame opens two or
                    // three tokens back.
                    if preposition_text.eq_ignore_ascii_case("aus")
                        && (2..=3).any(|back| {
                            index
                                .checked_sub(2 * back)
                                .and_then(word_at)
                                .as_deref()
                                .is_some_and(|word| word.eq_ignore_ascii_case("von"))
                        })
                    {
                        continue;
                    }
                }

                let suggestions: Vec<Suggestion> =
                    forms_in_case(&determiner_text, government.cases)
                        .into_iter()
                        .map(|form| Suggestion::ReplaceWith(form.chars().collect()))
                        .collect();

                if suggestions.is_empty() {
                    continue;
                }

                lints.push(Lint {
                    span: tokens[index + 2].span,
                    lint_kind: LintKind::Grammar,
                    suggestions,
                    message: format!(
                        "»{preposition_text}« verlangt {}. »{determiner_text}« steht {}.",
                        Self::label(government.cases, "den"),
                        Self::label(cases, "im")
                    ),
                    priority: 31,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Prüft, ob der Artikel nach einer Präposition im richtigen Fall steht."
    }
}

/// Whether `word` is an infinitive with `zu` written inside it, as a separable
/// verb requires: *entgegenzuwirken*, *vorzubeugen*, *zuzuschauen*.
///
/// The infix has to start at the third letter or later, so that *zusammen* —
/// which merely begins with the same two letters — is not mistaken for one.
/// *zuzu…* is the exception, where the separable prefix is itself `zu`.
fn is_infinitive_with_zu(word: &str) -> bool {
    if !word.ends_with("en") || is_capitalized(word) {
        return false;
    }

    word.starts_with("zuzu") || word.match_indices("zu").any(|(at, _)| at >= 2)
}

fn is_capitalized(word: &str) -> bool {
    word.chars().next().is_some_and(char::is_uppercase)
}

/// Whether `word` can end the noun phrase that a postposition attaches to.
///
/// A capitalized word is a noun — unless it only looks like one because it
/// opens the sentence, which `sentence_initial` reports.
fn closes_a_noun_phrase(word: &str, sentence_initial: bool) -> bool {
    const PRONOUNS: &[&str] = &[
        "ihm", "ihr", "ihnen", "mir", "dir", "uns", "euch", "sich", "wem", "mich", "dich", "ihn",
        "es",
    ];

    (is_capitalized(word) && !sentence_initial)
        || word == "und"
        || PRONOUNS.contains(&word)
        || determiner_readings(word).is_some()
        || preposition_government(word).is_some()
}

#[cfg(test)]
mod tests {
    use super::GermanPrepositionCase;
    use crate::document::Document;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::{Lint, Linter, Suggestion};

    fn lint(text: &str) -> Vec<Lint> {
        let dictionary = combined_german_dictionary();
        let document = Document::new_markdown_default(text, &dictionary);
        GermanPrepositionCase.lint(&document)
    }

    fn fixes(text: &str) -> Vec<String> {
        lint(text)
            .iter()
            .flat_map(|lint| &lint.suggestions)
            .map(|suggestion| match suggestion {
                Suggestion::ReplaceWith(chars) => chars.iter().collect(),
                _ => String::new(),
            })
            .collect()
    }

    fn assert_clean(texts: &[&str]) {
        for text in texts {
            let lints = lint(text);
            assert!(
                lints.is_empty(),
                "{text}\n{:?}",
                lints.iter().map(|l| &l.message).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn catches_the_dative_after_a_genitive_preposition() {
        assert_eq!(fixes("Wegen dem Wetter bleiben wir zu Hause."), ["des"]);
        assert_eq!(fixes("Trotz dem Regen gingen wir spazieren."), ["des"]);
    }

    #[test]
    fn catches_the_dative_after_waehrend() {
        assert_eq!(fixes("Während dem Essen sprach niemand."), ["des"]);
    }

    #[test]
    fn catches_the_dative_after_an_accusative_preposition() {
        assert_eq!(fixes("Das Geschenk ist für dem Kind."), ["das", "den"]);
        assert_eq!(fixes("Wir gingen ohne dem Hund los."), ["das", "den"]);
    }

    #[test]
    fn catches_the_nominative_after_an_accusative_preposition() {
        // *der* is masculine singular, feminine singular or plural, so the
        // accusative it should have been is *den* or *die*.
        assert_eq!(fixes("Er lief durch der Wald."), ["den", "die"]);
    }

    #[test]
    fn catches_the_accusative_after_a_dative_preposition() {
        assert_eq!(fixes("Sie fuhr mit das Auto zur Arbeit."), ["dem"]);
        assert_eq!(fixes("Wir kommen aus die Stadt."), ["den", "der"]);
    }

    #[test]
    fn catches_a_wrong_indefinite_article() {
        assert_eq!(fixes("Ich spreche mit eine Frau."), ["einer"]);
        assert_eq!(fixes("Das ist für einem Freund."), ["ein", "einen"]);
    }

    #[test]
    fn catches_a_wrong_possessive() {
        assert_eq!(fixes("Er kam mit seine Schwester."), ["seinen", "seiner"]);
        assert_eq!(fixes("Das ist für meinem Vorschlag."), ["mein", "meinen"]);
    }

    #[test]
    fn catches_a_wrong_demonstrative() {
        assert_eq!(fixes("Wegen diesem Problem kam er zu spät."), ["dieses"]);
    }

    #[test]
    fn keeps_the_capitalization_of_the_determiner() {
        // Only the determiner is replaced, so its own case is what matters;
        // the preposition in front of it may be sentence-initial.
        assert_eq!(fixes("Wegen dem Regen kam sie spät."), ["des"]);
    }

    #[test]
    fn accepts_correct_case_government() {
        assert_clean(&[
            "Sie fuhr mit dem Auto zur Arbeit.",
            "Wegen des Wetters bleiben wir zu Hause.",
            "Das Geschenk ist für das Kind.",
            "Er lief durch den Wald.",
            "Wir kommen aus der Stadt.",
            "Ich spreche mit einer Frau.",
            "Trotz des Regens gingen wir spazieren.",
            "Er kam mit seinem Bruder.",
        ]);
    }

    /// Both cases are standard after these, so neither can be an error.
    #[test]
    fn accepts_a_two_way_preposition_in_either_case() {
        assert_clean(&[
            "Das Buch liegt auf dem Tisch.",
            "Ich lege das Buch auf den Tisch.",
            "Wir gehen in die Schule.",
            "Wir sind in der Schule.",
            "Das Bild hängt über dem Sofa.",
        ]);
    }

    /// *während*, *statt* and *anstatt* also introduce clauses, whose subject
    /// is nominative and must not be read as a failed genitive.
    #[test]
    fn leaves_subordinate_clauses_alone() {
        assert_clean(&[
            "Während das Kind schlief, las sie ein Buch.",
            "Während die Gäste warteten, kochte er weiter.",
            "Statt das Fenster zu schließen, ging er hinaus.",
            "Anstatt die Arbeit zu beenden, ging er nach Hause.",
        ]);
    }

    /// A genitive in front of its noun is elevated but correct after any
    /// preposition.
    #[test]
    fn leaves_a_prenominal_genitive_alone() {
        assert_clean(&[
            "Er handelte in des Kaisers Namen.",
            "Sie stand an des Vaters Grab.",
        ]);
    }

    /// The case belongs to *willen*, not to *um*.
    #[test]
    fn leaves_the_circumposition_um_willen_alone() {
        assert_clean(&[
            "Um der Sache willen schwieg er.",
            "Er tat es um des Friedens willen.",
        ]);
    }

    #[test]
    fn leaves_fixed_expressions_alone() {
        assert_clean(&["Trotz allem blieb sie freundlich.", "Von wegen dem Chef!"]);
    }

    /// A capitalized determiner belongs to a name.
    #[test]
    fn leaves_a_capitalized_determiner_alone() {
        assert_clean(&["Ein Bericht aus Die Zeit von gestern."]);
    }

    /// Bare *ihr* after a preposition is the personal pronoun.
    #[test]
    fn leaves_the_pronoun_ihr_alone() {
        assert_clean(&["Ich gehe mit ihr ins Kino.", "Das Buch ist für ihr Kind."]);
    }

    /// The infinitive of a separable verb writes *zu* inside the word, so
    /// searching for it as a token is not enough.
    #[test]
    fn leaves_infinitive_clauses_alone() {
        assert_clean(&[
            "Um dem Problem zu begegnen, brauchen wir Geld.",
            "Er schwieg, um dem Gericht weitere Ermittlungen zu ermöglichen.",
            "Sie handelte, um einem Missverständnis vorzubeugen.",
            "Wir kamen, um dem Spiel zuzuschauen.",
            "Er ging, ohne dem Gegner Gelegenheit zur Stellungnahme zu geben.",
            "Anstatt den Strahlungsfluss direkt zu messen, wird verglichen.",
        ]);
    }

    /// *nach*, *zufolge*, *gegenüber* and *aus* follow their noun phrase at
    /// least as often as they precede it, and then govern nothing to their
    /// right.
    #[test]
    fn leaves_postpositions_alone() {
        assert_clean(&[
            "Seiner Ansicht nach eine gute Lösung.",
            "Das Kind lernt nach und nach die Grammatik.",
            "Einer Auffassung zufolge ein mögliches Indiz für Bewusstsein.",
            "Allen anderen Menschen gegenüber eine Überlegenheit.",
            "Von dort aus eine Stunde zu Fuß.",
            "Von wo aus eine Aussage gültig ist.",
            "Von wegen dem Chef!",
        ]);
    }

    /// A determiner in a fixed coordination stays uninflected.
    #[test]
    fn leaves_fixed_coordinations_alone() {
        assert_clean(&[
            "Das Haus wurde von ein und demselben Architekten gebaut.",
            "Die Leitung wird von ein oder mehreren Seilen überspannt.",
            "Antikörper entstehen innerhalb von ein bis zwei Wochen.",
        ]);
    }

    /// Every one of these is a noun or an acronym here, not the preposition it
    /// is spelled like. A German preposition is lower case unless it opens the
    /// sentence.
    #[test]
    fn leaves_capitalized_homographs_alone() {
        assert_clean(&[
            "Er bestieg für kurze Zeit den Thron.",
            "Am MIT das erste Betriebssystem zu bauen war schwierig.",
            "Sun Quan gewährte Liu Bei die Provinz Jingzhou.",
            "Die Einkehr in eine Bar eine der wenigen Möglichkeiten.",
        ]);
    }

    /// Both are conjunctions often enough that they are out of the table.
    #[test]
    fn leaves_bis_and_seit_alone() {
        assert_clean(&[
            "Wir warten, bis der Zug kommt.",
            "Seit das Kind hier ist, schlafen wir weniger.",
        ]);
    }

    /// The rule sees the determiner alone, so a form that is right in some
    /// reading passes even when the noun proves it wrong. Recorded so the
    /// limit is not mistaken for a bug.
    #[test]
    fn misses_what_the_determiner_alone_does_not_reveal() {
        assert_clean(&[
            // *den* is a perfectly good dative plural, and only the singular
            // *Freund* gives the mistake away.
            "Ich gehe mit den Freund ins Kino.",
            // *seinen* likewise: accusative singular, but also dative plural.
            "Er kam mit seinen Bruder.",
        ]);
    }

    #[test]
    fn needs_a_noun_after_the_determiner() {
        assert_clean(&["Er entschied sich für das."]);
    }

    #[test]
    fn ignores_a_preposition_at_the_end_of_a_sentence() {
        assert_clean(&["Damit rechnet er nicht mit"]);
    }
}
