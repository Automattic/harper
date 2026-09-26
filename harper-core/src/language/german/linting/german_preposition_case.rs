//! Checks the case of a determiner directly after a preposition.

use std::sync::Arc;

use crate::{
    Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::grammar::determiners::{
        DeterminerReading, could_be_a_dative_plural, determiner_readings, forms_for_readings,
        readings_allowed_by, readings_allowed_by_spelling, stands_alone,
    },
    language::german::grammar::prepositions::preposition_government,
    language::german::spell::curated_german_dictionary,
    language::morphology::{Agreement, CaseSet, MorphologyExt},
    linting::{Lint, LintKind, Linter, Suggestion},
    spell::{Dictionary, FstDictionary},
};

/// How many words may stand between the determiner and its noun.
///
/// *mit dem sehr alten Haus* is three; beyond that the phrase has almost
/// certainly ended and the capitalized word belongs to something else.
const MAX_ADJECTIVES: usize = 3;

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
/// The noun narrows the determiner before the intersection is taken. *den* is
/// accusative masculine singular or dative plural, and nothing about the word
/// says which; *Freund* is singular, so the dative plural reading cannot stand
/// and *mit den Freund* has no dative left. An axis the dictionary is silent
/// about narrows nothing, so this only ever adds catches.
///
/// What is still out of reach is a mistake that survives both: *mit den Lehrer*
/// is wrong, but *Lehrer* is one form for the singular and the plural, and only
/// its missing dative plural `-n` gives it away. That needs case on the nouns,
/// which the dictionary does not carry.
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
pub struct GermanPrepositionCase {
    /// The base dictionary, deliberately not the compound-aware one: a
    /// compound decomposition invents readings, and a wrong gender on the noun
    /// would rule out a reading the determiner really has and turn a correct
    /// phrase into a lint.
    dictionary: Arc<FstDictionary>,
}

impl Default for GermanPrepositionCase {
    fn default() -> Self {
        Self::new()
    }
}

impl GermanPrepositionCase {
    pub fn new() -> Self {
        Self {
            dictionary: curated_german_dictionary(),
        }
    }

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

impl GermanPrepositionCase {
    /// The head of the noun phrase this determiner introduces, if it can be
    /// identified with confidence.
    ///
    /// German capitalizes its nouns, which makes the head easy to find in the
    /// ordinary case: skip the lower-case adjectives and take the first
    /// capitalized word. Three things spoil that, and each cost false positives
    /// on the prose corpus before it was turned away:
    ///
    /// * **A hyphen.** In *aus den Natur- und Geisteswissenschaften* and *zu
    ///   den Absinth-Trinkern* the first capitalized word is half of a compound
    ///   and carries none of the phrase's features.
    /// * **A phrase that has not ended.** A following capitalized word (*mit
    ///   den Florida Keys*) or a following adjective (*bei den lange Zeit
    ///   allein bekannten Verfahren*) both say the head is further on.
    /// * **A determiner, preposition or conjunction along the way.** *mit
    ///   diesen in Konkurrenz* has no noun of its own; the determiner is a
    ///   pronoun and the capitalized word belongs to what follows.
    fn head_noun_after(
        &self,
        document: &Document,
        tokens: &[&Token],
        determiner_at: usize,
    ) -> Option<String> {
        let content = document.get_full_content();
        let word_at = |at: usize| -> Option<String> {
            tokens
                .get(at)
                .filter(|token| matches!(token.kind, TokenKind::Word(_)))
                .map(|token| document.get_span_content(&token.span).iter().collect())
        };

        let mut at = determiner_at + 2;
        for _ in 0..=MAX_ADJECTIVES {
            let word = word_at(at)?;

            if !is_capitalized(&word) {
                if closes_the_phrase(&word) {
                    return None;
                }
                at += 2;
                continue;
            }

            let token = tokens[at];
            let touches_hyphen = content.get(token.span.end) == Some(&'-')
                || (token.span.start > 0 && content.get(token.span.start - 1) == Some(&'-'));
            if touches_hyphen {
                return None;
            }

            let ends_here = match word_at(at + 2) {
                None => tokens
                    .get(at + 2)
                    .is_none_or(|next| !matches!(next.kind, TokenKind::Word(_))),
                Some(next) => !is_capitalized(&next) && closes_the_phrase(&next),
            };
            return ends_here.then_some(word);
        }

        None
    }

    /// What the dictionary says about `head`, when that is worth reading.
    ///
    /// A form ending in `-n` or `-s` is held back. Every German dative plural
    /// ends in `-n`, and most of them are missing from the dictionary —
    /// *Fischen*, *Berufen*, *Zeichen*, *Männchen* all resolve to an entry
    /// recorded as a singular, and narrowing by that would invent errors. The
    /// spelling of those forms is read instead, by
    /// [`readings_allowed_by_spelling`], which asks the opposite question and
    /// needs no entry at all.
    fn agreement_of(&self, head: &str) -> Agreement {
        if could_be_a_dative_plural(head) {
            return Agreement::default();
        }

        let chars: Vec<char> = head.chars().collect();
        self.dictionary
            .get_word_metadata(&chars)
            .filter(|metadata| metadata.is_noun())
            .map(|metadata| metadata.noun_agreement())
            .unwrap_or_default()
    }
}

/// Contractions of a preposition and an article, which the tokenizer keeps
/// whole and which therefore never reach the preposition table.
const CONTRACTIONS: &[&str] = &[
    "im", "am", "zum", "zur", "beim", "vom", "ins", "ans", "aufs", "durchs", "fürs", "ums",
    "übers", "unters", "hinters", "vors",
];

/// Whether `word` ends the noun phrase in front of it rather than continuing
/// it. A determiner opens a new one, a preposition or a conjunction closes the
/// old one, and an adjective does neither.
fn closes_the_phrase(word: &str) -> bool {
    const CONJUNCTIONS: &[&str] = &["und", "oder", "sowie", "aber", "denn", "sondern", "als"];

    determiner_readings(word).is_some()
        || preposition_government(word).is_some()
        || CONTRACTIONS.contains(&word)
        || CONJUNCTIONS.contains(&word)
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

                let Some(all_readings) = determiner_readings(&determiner_text) else {
                    continue;
                };

                // The noun rules out the readings it cannot stand beside. Its
                // spelling does most of the work — a German dative plural ends
                // in `-n`, so *mit den Freund* has no dative left whatever the
                // dictionary knows — and its entry adds what it can.
                // A head that cannot be identified narrows nothing; the check
                // still runs on the determiner alone, which is what it did
                // before the noun was read at all.
                let head = self
                    .head_noun_after(document, &tokens, index + 2)
                    .filter(|_| !stands_alone(&determiner_text));
                let readings: Vec<DeterminerReading> = match &head {
                    None => all_readings.to_vec(),
                    Some(head) => readings_allowed_by(
                        &readings_allowed_by_spelling(all_readings, head),
                        &self.agreement_of(head),
                    ),
                };
                let cases = readings
                    .iter()
                    .fold(CaseSet::empty(), |set, reading| set | reading.case.into());

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

                let suggestions: Vec<Suggestion> = forms_for_readings(&readings, government.cases)
                    .into_iter()
                    .filter(|form| *form != determiner_text)
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
        GermanPrepositionCase::new().lint(&document)
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
        // *dem* is singular in both its readings, so the noun rules nothing
        // out and both genders are offered. Narrowing to the masculine would
        // need gender, which the dictionary gets wrong too often to use.
        assert_eq!(fixes("Das Geschenk ist für dem Kind."), ["das", "den"]);
        assert_eq!(fixes("Wir gingen ohne dem Hund los."), ["das", "den"]);
    }

    #[test]
    fn catches_the_nominative_after_an_accusative_preposition() {
        // *Wald* is singular, which drops the genitive plural reading of
        // *der*; the two singular genders remain.
        assert_eq!(fixes("Er lief durch der Wald."), ["den", "die"]);
    }

    #[test]
    fn catches_the_accusative_after_a_dative_preposition() {
        assert_eq!(fixes("Sie fuhr mit das Auto zur Arbeit."), ["dem"]);
        // *Stadt* is a feminine singular, so the dative plural *den* is not
        // offered.
        assert_eq!(fixes("Wir kommen aus die Stadt."), ["der"]);
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

    /// The determiner alone says nothing here — *den* is a perfectly good
    /// dative plural. The noun is what settles it.
    #[test]
    fn the_noun_reveals_what_the_determiner_hides() {
        assert_eq!(fixes("Ich gehe mit den Freund ins Kino."), ["dem"]);
        assert_eq!(fixes("Sie spielt mit den Hund im Garten."), ["dem"]);
        // A two-way preposition governs the accusative too, so this is right.
        assert_clean(&["Er antwortet auf den Brief nicht."]);
    }

    /// The noun's *spelling* settles what its entry cannot. *Lehrer* is one
    /// form for the singular and the plural, so its number says nothing — but
    /// every German dative plural ends in `-n`, and *Lehrern* is the one that
    /// would. No entry has to carry a number, a gender or a case for this,
    /// which is as well: `bruder` and `zug` carry none of the three.
    #[test]
    fn the_spelling_of_the_noun_rules_out_a_dative_plural() {
        assert_eq!(fixes("Ich spreche mit den Lehrer über die Note."), ["dem"]);
        assert_eq!(fixes("Er kam mit seinen Bruder zum Essen."), ["seinem"]);
        assert_eq!(fixes("Wir fahren mit den Zug nach Berlin."), ["dem"]);
        assert_eq!(fixes("Er arbeitet bei den Bäcker."), ["dem"]);
    }

    /// And a noun that really could be a dative plural keeps the reading.
    #[test]
    fn a_noun_that_could_be_a_dative_plural_is_left_alone() {
        assert_clean(&[
            "Ich spreche mit den Lehrern über die Note.",
            "Er kam mit seinen Brüdern zum Essen.",
            "Wir fahren mit den Autos nach Berlin.",
            // A Latin plural takes no `-n`: *den Korpora*, *den Termini*.
            "Zu den Korpora des Altenglischen gehört dieser Bestand.",
            "Im Vergleich zu den Termini der Logik ist das einfach.",
            // An acronym inflects for nothing.
            "Bei den NSAR ist die Wirkung entzündungsmindernd.",
        ]);
    }

    /// *Zu diesen zählen Annegray und Luxeuil*: the determiner is the whole
    /// phrase and the capitalized word belongs to what follows. The article
    /// forms are not like this, so only these five are held back.
    #[test]
    fn a_freely_pronominal_determiner_is_not_given_a_noun() {
        assert_clean(&[
            "Zu diesen zählen Annegray, Luxeuil und St. Gallen.",
            "Im Vergleich zu jenen künstlicher Texte ist das anders.",
            "Bei welchen Hinweise vorhanden sind, ist unklar.",
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
