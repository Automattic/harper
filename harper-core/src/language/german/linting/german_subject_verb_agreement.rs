//! The verb has to match the subject in person and number.

use std::sync::Arc;

use crate::language::german::grammar::subjects::{
    Features, irregular_finite_verb, subject_pronoun, subordinate_subject_pronoun,
};
use crate::language::morphology::{MorphologyExt, NumberSet, PersonSet};
use crate::linting::{Lint, LintKind, Linter};
use crate::spell::Dictionary;
use crate::{Punctuation, Token, TokenKind, TokenStringExt, document::Document};

/// Requires a finite verb to agree with the pronoun in front of it.
///
/// German marks person and number on the verb, so *er gehen* and *du hat* are
/// wrong in a way no amount of context can rescue. This is the one agreement
/// class Harper can decide without gender, and the features come from two
/// places, both of them complete:
///
/// * the **subject** is a personal pronoun, a closed class of eight forms, in
///   `grammar/subjects.rs`.
/// * the **verb** carries person and number from its conjugation affix
///   (`annotations.json`), or, when it is irregular enough that the dictionary
///   stores the whole form, from the table beside the pronouns.
///
/// Two positions are checked, and they are the two where the finite verb can
/// be *located* without parsing the sentence:
///
/// * the **front field**. German is verb-second, so when the pronoun opens its
///   clause the finite verb is the next word. Nowhere else in a main clause is
///   that true — in *das er vergessen hat* the word behind the pronoun is a
///   participle — and checking the other pairs reported eight hundred times on
///   edited prose and was wrong every time.
/// * the **end of a subordinate clause**. A clause opened by *dass*, *weil* or
///   *wie* is verb-final, so the last word before the clause's punctuation is
///   the verb. This is the half of the language the front field cannot see, and
///   LanguageTool does not see it either: its `DE_VERBAGREEMENT` wants the two
///   words adjacent, so *Es ist nicht so wie es sein sollt* goes unreported by
///   both tools until now.
///
/// The second position is worth less than it looks, because `-en` has to be
/// thrown away there — it is the infinitive, the plural and the declined
/// adjective at once. What is left is the irregular auxiliaries and modals,
/// which is where the frequency is anyway.
///
/// A **noun-phrase subject** — *die Kinder spielt* — is not checked, and the
/// attempt is worth recording. Finding the head is easy enough, but the word
/// behind it is not reliably the verb: *die Gesellschaft bürgerlichen Rechts*
/// and *die Arten hohler Stängel* put an adjective there, and a relative clause
/// behind a comma (*…, welches Sittenwidrigkeit impliziert*) passes the
/// front-field test while being verb-final. That version reported 1229 times on
/// the same prose. It needs the noun-phrase chunker the capitalization rule
/// has, not another guard.
/// Conjunctions that put their clause in verb-final order.
///
/// Only the unambiguous ones, and only those that can be followed directly by
/// a pronoun subject. A word that is also a question word (*wie*, *wo*) is
/// still safe here, because the check needs a pronoun immediately behind it
/// and a question puts the verb there instead: *wie geht es* never matches,
/// *wie es sein soll* does.
const SUBORDINATORS: &[&str] = &[
    "dass",
    "weil",
    "wenn",
    "ob",
    "obwohl",
    "obgleich",
    "obschon",
    "sodass",
    "damit",
    "falls",
    "sobald",
    "solange",
    "seitdem",
    "nachdem",
    "bevor",
    "während",
    "zumal",
    "sofern",
    "soweit",
    "wohingegen",
    "indem",
    "wenngleich",
    "da",
    "wie",
    "wo",
];

/// Coordinating conjunctions, which end the clause for this purpose.
///
/// *da ich keine Beweise hätte und der Dieb es nicht zurückgeben wird* puts a
/// second clause with a subject of its own behind the *und*, and the word at
/// the end belongs to that one. Stopping early can only cost a detection,
/// never cause a report: what it hands back instead is a word from the middle
/// of a phrase, which the two filters below throw away.
const COORDINATORS: &[&str] = &["und", "oder", "aber", "sondern", "denn", "doch", "sowie"];

pub struct GermanSubjectVerbAgreement<T>
where
    T: Dictionary,
{
    dictionary: Arc<T>,
}

impl<T: Dictionary> GermanSubjectVerbAgreement<T> {
    pub fn new(dictionary: Arc<T>) -> Self {
        Self { dictionary }
    }

    /// The readings of a finite verb form, as *joint* person/number pairs.
    ///
    /// A list rather than two sets, and the reason is the `-t` ending: it is
    /// third person singular and second person plural, and independent axes
    /// would also admit third person plural, which is exactly the reading *die
    /// Kinder spielt* needs ruled out. The determiner table next door keeps its
    /// readings joint for the same reason.
    ///
    /// The dictionary says *whether* the word is a finite form and roughly
    /// which features it has; the ending says how they pair up. Nothing else
    /// can: an affix rule carries one metadata block for all its replacements.
    ///
    /// The irregular table wins over the dictionary: *ist*, *hat* and *sind*
    /// exist as entries of their own and carry generic verb flags, some of
    /// which are also affixes and would hand back a reading built for a
    /// different word.
    fn verb_features(&self, word: &str, token: &Token) -> Vec<Features> {
        if let Some(features) = irregular_finite_verb(word) {
            return vec![features];
        }

        // The affix-built forms. What identifies them is the person and number
        // the conjugation affix left behind, not a part of speech: the
        // preterite affixes sit on two thousand noun and adjective entries as
        // well, so they cannot declare their output a verb without turning
        // those into verbs too.
        //
        // What the part of speech does here is rule things out. The `-st`
        // affix is applied to pronoun and adjective roots, so `selbst` and
        // `möglichst` arrive carrying a second person singular; a determiner, a
        // pronoun or an adverb is not the finite verb of the clause. The
        // article `die` used to need this too, until the root that built it
        // lost the flag — see `tests/verb_person_test.rs`.
        if token.kind.is_determiner() || token.kind.is_pronoun() {
            return Vec::new();
        }

        // A finite verb is never capitalized mid-sentence, and the pronoun in
        // front of this one guarantees we are mid-sentence. What a capital
        // marks here is an apposition — *wir Arbeiter*, *wir Deutsche* — whose
        // head is a noun that happens to carry a conjugation affix.
        if word.chars().next().is_some_and(char::is_uppercase) {
            return Vec::new();
        }

        let chars: Vec<char> = word.chars().collect();
        let Some(metadata) = self.dictionary.get_word_metadata(&chars) else {
            return Vec::new();
        };

        // An adverb is not the finite verb, whatever else the entry says. The
        // `-st` affix is applied to adjective and pronoun roots as well as to
        // verbs, so `selbst` and `möglichst` arrive here carrying a second
        // person singular; sixteen of the seventeen reports left on the prose
        // corpus were that one ending.
        if metadata.is_adverb() {
            return Vec::new();
        }

        let agreement = metadata.verb_agreement();
        if agreement.person.is_empty() || agreement.number.is_empty() {
            return Vec::new();
        }

        readings_of_ending(word)
    }

    /// Does the token at `index` stand in the front field of its clause?
    ///
    /// That means the start of the sentence, or directly behind a comma or a
    /// coordinating conjunction — the positions from which German's verb-second
    /// rule puts the finite verb next. Behind a subordinator or a relative
    /// pronoun the clause is verb-final instead and the next word is anything
    /// but the finite verb.
    fn opens_a_clause(tokens: &[&Token], index: usize, document: &Document) -> bool {
        const COORDINATORS: &[&str] = &["und", "oder", "aber", "denn", "sondern", "doch"];

        let Some(previous) = index.checked_sub(1).map(|i| tokens[i]) else {
            return true;
        };

        match previous.kind {
            TokenKind::Punctuation(
                Punctuation::Comma | Punctuation::Semicolon | Punctuation::Colon,
            ) => true,
            TokenKind::Word(_) => {
                let word: String = document
                    .get_span_content(&previous.span)
                    .iter()
                    .flat_map(|c| c.to_lowercase())
                    .collect();
                COORDINATORS.contains(&word.as_str())
            }
            _ => false,
        }
    }

    /// The word at `index`, lower-cased.
    fn word_at(tokens: &[&Token], index: usize, document: &Document) -> String {
        document
            .get_span_content(&tokens[index].span)
            .iter()
            .flat_map(|c| c.to_lowercase())
            .collect()
    }

    /// The finite verb of a subordinate clause opened at `index`, if it can be
    /// found without guessing.
    ///
    /// German puts the finite verb last in a subordinate clause, which is the
    /// one other position where it can be located by counting rather than by
    /// parsing. The clause runs from the subject to the first punctuation mark
    /// or coordinating conjunction; its last word is the verb.
    ///
    /// The exception is the *Ersatzinfinitiv* — *dass er hat kommen können* —
    /// where the auxiliary leads a cluster of two bare infinitives instead of
    /// standing last. Two `-en` forms in a row is the shape of it, and also the
    /// shape of *dass wir gelesen haben*, where the last word really is the
    /// verb; both are skipped, because for a plural subject the reading would
    /// have agreed anyway.
    fn clause_final_verb(tokens: &[&Token], index: usize, document: &Document) -> Option<usize> {
        let mut words: Vec<usize> = Vec::new();

        for position in index..tokens.len() {
            match tokens[position].kind {
                TokenKind::Word(_) => {
                    if Self::word_ends_clause(tokens, position, document) {
                        break;
                    }
                    words.push(position);
                }
                TokenKind::Punctuation(Punctuation::Hyphen | Punctuation::Apostrophe) => {}
                TokenKind::Punctuation(_) => break,
                _ => {}
            }
        }

        let last = *words.last()?;

        // `-en` is banned in this position, and it is the whole reason the
        // front-field check was written first. The ending is the infinitive,
        // the first and third person plural, and the declined adjective, all
        // three — and clause-finally German uses every one of them. *dass er
        // versucht zu schlafen* extraposes an infinitive past the finite verb,
        // *damit er handeln können* belongs to a plural subject further out,
        // and *da er es in einem konkreten, empirischen Zusammenhang sieht*
        // ends a fragment on an adjective. All twenty reports the first version
        // produced on edited prose ended in `-en`.
        //
        // In the front field the position itself guaranteed a finite verb and
        // the ending could be trusted. Here nothing does, so the check keeps
        // only the endings an infinitive cannot wear.
        if Self::word_at(tokens, last, document).ends_with("en") {
            return None;
        }

        // And an adjective is not the verb either. The `-e` affix builds
        // *positive* as readily as *lerne*, and stopping at a coordinator can
        // leave the scan standing on one: *ob wir eine positive oder negative
        // Wirkung erwarten*. The front field never needed this, because a
        // pronoun is not followed by an attributive adjective.
        if tokens[last].kind.is_adjective() {
            return None;
        }

        Some(last)
    }

    fn word_ends_clause(tokens: &[&Token], index: usize, document: &Document) -> bool {
        COORDINATORS.contains(&Self::word_at(tokens, index, document).as_str())
    }

    fn report(subject_text: &str, subject: &Features, verb_text: &str, verb: &Token) -> Lint {
        Lint {
            span: verb.span,
            lint_kind: LintKind::Agreement,
            suggestions: Vec::new(),
            priority: 30,
            message: format!(
                "»{subject_text}« verlangt {}. »{verb_text}« steht in einer anderen Form.",
                Self::wanted(subject)
            ),
        }
    }

    /// The second position where the finite verb can be found by counting.
    ///
    /// A subordinate clause is verb-final, so *weil du kommen sollt* pairs the
    /// pronoun behind the conjunction with the last word of the clause. This is
    /// the half of the language the front-field check cannot see, and neither
    /// tool Harper is measured against sees it either: LanguageTool's
    /// `DE_VERBAGREEMENT` wants the two words adjacent.
    ///
    /// `es` is admitted here and nowhere else — see
    /// [`subordinate_subject_pronoun`] — but only its **person** is checked.
    /// The number comes from the predicate in *weil es meine Freunde sind*, and
    /// no third-person form is ever wrong after *es*. What is left is the
    /// second person, which *es* can never take: *wie es sein sollt*.
    fn lint_subordinate_clauses(
        &self,
        tokens: &[&Token],
        document: &Document,
        lints: &mut Vec<Lint>,
    ) {
        for index in 0..tokens.len().saturating_sub(2) {
            if !matches!(tokens[index].kind, TokenKind::Word(_))
                || !SUBORDINATORS.contains(&Self::word_at(tokens, index, document).as_str())
            {
                continue;
            }

            let subject_index = index + 1;
            if !matches!(tokens[subject_index].kind, TokenKind::Word(_)) {
                continue;
            }

            let subject_text: String = document
                .get_span_content(&tokens[subject_index].span)
                .iter()
                .collect();
            let Some(subject) = subordinate_subject_pronoun(&subject_text) else {
                continue;
            };

            let Some(verb_index) = Self::clause_final_verb(tokens, subject_index + 1, document)
            else {
                continue;
            };

            let verb_token = tokens[verb_index];
            let verb_text: String = document.get_span_content(&verb_token.span).iter().collect();
            let readings = self.verb_features(&verb_text, verb_token);
            if readings.is_empty() {
                continue;
            }

            let placeholder = subject_text.eq_ignore_ascii_case("es");
            let agrees = |reading: &Features| {
                if placeholder {
                    subject.person.agrees_with(reading.person)
                } else {
                    subject.agrees_with(reading)
                }
            };
            if readings.iter().any(agrees) {
                continue;
            }

            lints.push(Self::report(
                &subject_text,
                &subject,
                &verb_text,
                verb_token,
            ));
        }
    }

    /// Which form the subject wants, spelled out for the message.
    fn wanted(features: &Features) -> &'static str {
        use crate::language::morphology::{NumberSet, PersonSet};
        match (features.person, features.number) {
            (PersonSet::FIRST, NumberSet::SINGULAR) => "die 1. Person Singular",
            (PersonSet::SECOND, NumberSet::SINGULAR) => "die 2. Person Singular",
            (PersonSet::THIRD, NumberSet::SINGULAR) => "die 3. Person Singular",
            (PersonSet::FIRST, NumberSet::PLURAL) => "die 1. Person Plural",
            (PersonSet::SECOND, NumberSet::PLURAL) => "die 2. Person Plural",
            (PersonSet::THIRD, NumberSet::PLURAL) => "die 3. Person Plural",
            _ => "eine andere Form",
        }
    }
}

impl<T: Dictionary> Linter for GermanSubjectVerbAgreement<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let tokens: Vec<&Token> = sentence
                .iter()
                .filter(|token| !token.kind.is_whitespace())
                .collect();

            for index in 0..tokens.len().saturating_sub(1) {
                if !Self::opens_a_clause(&tokens, index, document) {
                    continue;
                }

                let (subject_token, verb_token) = (tokens[index], tokens[index + 1]);
                if !matches!(subject_token.kind, TokenKind::Word(_))
                    || !matches!(verb_token.kind, TokenKind::Word(_))
                {
                    continue;
                }

                let subject_text: String = document
                    .get_span_content(&subject_token.span)
                    .iter()
                    .collect();
                let Some(subject) = subject_pronoun(&subject_text) else {
                    continue;
                };

                let verb_text: String =
                    document.get_span_content(&verb_token.span).iter().collect();
                let verb = self.verb_features(&verb_text, verb_token);
                if verb.is_empty() || verb.iter().any(|reading| subject.agrees_with(reading)) {
                    continue;
                }

                lints.push(Self::report(
                    &subject_text,
                    &subject,
                    &verb_text,
                    verb_token,
                ));
            }

            self.lint_subordinate_clauses(&tokens, document, &mut lints);
        }

        lints
    }

    fn description(&self) -> &str {
        "Prüft, ob das Verb in Person und Numerus zum Subjekt passt."
    }
}

/// How a conjugation ending pairs person with number.
///
/// The endings are checked longest first, because `-ten` is an `-en` and `-st`
/// is not a `-t`. Every pair here is a reading the ending genuinely has; the
/// caller has already established from the affix metadata that the word is a
/// finite form at all, which is what keeps a noun ending in `-t` out.
fn readings_of_ending(word: &str) -> Vec<Features> {
    let first_singular = Features::new(PersonSet::FIRST, NumberSet::SINGULAR);
    let third_singular = Features::new(PersonSet::THIRD, NumberSet::SINGULAR);
    let second_singular = Features::new(PersonSet::SECOND, NumberSet::SINGULAR);
    let first_plural = Features::new(PersonSet::FIRST, NumberSet::PLURAL);
    let second_plural = Features::new(PersonSet::SECOND, NumberSet::PLURAL);
    let third_plural = Features::new(PersonSet::THIRD, NumberSet::PLURAL);

    if word.ends_with("st") {
        // du lernst, du arbeitest — and the third person as well, because a
        // verb whose stem ends in a sibilant spells both the same way: *du
        // weist* and *er weist*, *du misst* and *er misst*, *du liest* and *er
        // liest*. Nothing on the surface separates `weis` + `t` from `lern` +
        // `st`, so the third person stays in. The cost is that *er lernst* goes
        // unreported; the gain is the twenty-five reports that `weist`,
        // `verweist`, `misst`, `fasst` and `liest` produced on edited prose.
        vec![second_singular, third_singular]
    } else if word.ends_with("en") {
        // wir/sie lernen, wir/sie lernten — and the infinitive, which has no
        // person at all and is why only the front field is checked.
        vec![first_plural, third_plural]
    } else if word.ends_with('t') {
        // er lernt, ihr lernt
        vec![third_singular, second_plural]
    } else if word.ends_with('e') {
        // ich lerne, er lerne (Konjunktiv I), ich/er lernte
        vec![first_singular, third_singular]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::GermanSubjectVerbAgreement;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn lint_count(text: &str) -> usize {
        let dictionary = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dictionary);
        GermanSubjectVerbAgreement::new(dictionary.clone())
            .lint(&document)
            .len()
    }

    /// The auxiliaries, which carry most of the German verb system and are the
    /// forms the conjugation affixes cannot build.
    #[test]
    fn an_auxiliary_has_to_match() {
        for text in [
            "Wir ist heute sehr müde.",
            "Du hat das Buch schon gelesen.",
            "Ich sind bereit.",
            "Er haben das Auto gekauft.",
            "Du bin zu spät.",
        ] {
            assert_eq!(lint_count(text), 1, "should fire on {text:?}");
        }
    }

    #[test]
    fn a_correct_auxiliary_is_quiet() {
        for text in [
            "Wir sind heute sehr müde.",
            "Du hast das Buch schon gelesen.",
            "Ich bin bereit.",
            "Er hat das Auto gekauft.",
            "Ihr seid zu spät.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// The regular endings, which come from the conjugation affixes.
    #[test]
    fn a_regular_ending_has_to_match() {
        for text in [
            "Er gehen jeden Tag nach Hause.",
            "Ich lernen Deutsch.",
            "Wir lernt Deutsch.",
            "Du spielen gern Klavier.",
        ] {
            assert_eq!(lint_count(text), 1, "should fire on {text:?}");
        }
    }

    #[test]
    fn a_correct_regular_ending_is_quiet() {
        for text in [
            "Er geht jeden Tag nach Hause.",
            "Ich lerne Deutsch.",
            "Wir lernen Deutsch.",
            "Du spielst gern Klavier.",
            "Sie spielt gern Klavier.",
            "Sie spielen gern Klavier.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// *sie* is third person in both numbers, so neither verb is wrong.
    #[test]
    fn sie_takes_either_number() {
        assert_eq!(lint_count("Sie ist müde."), 0);
        assert_eq!(lint_count("Sie sind müde."), 0);
        assert_eq!(lint_count("Sie bin müde."), 1, "but not the first person");
    }

    /// Konjunktiv I is spelled like the first person, and German prose reports
    /// speech constantly. The `-e` ending allows the third person for this.
    #[test]
    fn reported_speech_is_not_a_disagreement() {
        for text in [
            "Er sagte, er lerne viel.",
            "Sie erklärte, er komme später.",
            "Man sagte, er habe recht.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// Only the front field is checked. In a verb-final clause the word behind
    /// the pronoun is a participle or an infinitive, not the finite verb.
    #[test]
    fn a_verb_final_clause_is_left_alone() {
        for text in [
            "Das ist etwas, das er vergessen hat.",
            "Er blieb zu Hause, weil er gehen musste.",
            "Die Frage, die er stellen wollte, blieb offen.",
            "Ich weiß, dass er kommen wird.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// A coordinated main clause opens a front field of its own.
    #[test]
    fn a_coordinated_clause_is_checked() {
        assert_eq!(lint_count("Sie kam nach Hause und er gehen weg."), 1);
        assert_eq!(lint_count("Sie kam nach Hause und er ging weg."), 0);
    }

    /// The placeholder *es* is out of the table: the verb agrees with the noun
    /// behind it, not with the pronoun.
    #[test]
    fn the_placeholder_es_is_not_a_subject() {
        for text in [
            "Es werden fünf Klassen gebildet.",
            "Es existieren zahlreiche Ansätze.",
            "Es müssen viele Fragen geklärt werden.",
            "Es ist spät.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// `ihr` is a possessive and a dative far more often than a subject.
    #[test]
    fn ihr_is_not_read_as_a_subject() {
        for text in ["Ihr Auto ist neu.", "Ich gebe ihr Geld.", "Ihr habt recht."] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// An adverb is not the finite verb, whatever the `-st` affix produced.
    #[test]
    fn an_adverb_behind_the_pronoun_is_not_a_verb() {
        for text in [
            "Er selbst sprach von einem Paradigma.",
            "Sie selbst stellen eine wichtige Nahrung dar.",
            "Man selbst ist dafür verantwortlich.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// An oblique pronoun is not a subject, so nothing is compared.
    #[test]
    fn an_oblique_pronoun_starts_nothing() {
        for text in ["Ihn kennen wir gut.", "Mir ist kalt.", "Uns geht es gut."] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// The modals, which are the other half of the irregular table.
    #[test]
    fn the_modals_are_covered() {
        assert_eq!(lint_count("Du kann das nicht wissen."), 1);
        assert_eq!(lint_count("Du kannst das nicht wissen."), 0);
        assert_eq!(lint_count("Wir muss jetzt gehen."), 1);
        assert_eq!(lint_count("Wir müssen jetzt gehen."), 0);
    }

    /// The preterite splits by number the same way the present does.
    #[test]
    fn the_preterite_is_checked() {
        assert_eq!(lint_count("Wir lernte Deutsch."), 1);
        assert_eq!(lint_count("Wir lernten Deutsch."), 0);
        assert_eq!(lint_count("Ich lernten Deutsch."), 1);
        assert_eq!(lint_count("Ich lernte Deutsch."), 0);
    }

    /// The readings are joint pairs, not two independent sets, so the `-t`
    /// ending's two readings exclude the second person singular between them.
    #[test]
    fn the_endings_readings_are_joint() {
        assert_eq!(lint_count("Du lernt Deutsch."), 1, "-t is 3rd sg or 2nd pl");
        assert_eq!(lint_count("Ihr lernt Deutsch."), 0);
        assert_eq!(lint_count("Er lernt Deutsch."), 0);
    }

    /// A stem ending in a sibilant spells the second and third person alike,
    /// and nothing on the surface separates `weis` + `t` from `lern` + `st`.
    #[test]
    fn a_sibilant_stem_keeps_both_persons() {
        for text in [
            "Er weist darauf hin.",
            "Sie misst die Strecke.",
            "Er liest ein Buch.",
            "Du liest ein Buch.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
        assert_eq!(
            lint_count("Wir lernst Deutsch."),
            1,
            "the number still separates them"
        );
    }

    /// A word that is not in either table produces nothing at all.
    #[test]
    fn an_unknown_verb_is_not_guessed_at() {
        assert_eq!(lint_count("Er fnordet durch die Gegend."), 0);
    }

    /// An apposition behind the pronoun is a noun, and a capital says so.
    #[test]
    fn an_apposition_is_not_a_verb() {
        for text in [
            "Wir Deutsche reden gern über das Wetter.",
            "Wir Arbeiter haben andere Sorgen.",
            "Wir Frauen wissen das.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// The subordinate clause is verb-final, which is the second position
    /// where the finite verb can be found by counting rather than parsing.
    #[test]
    fn a_verb_final_clause_has_to_agree() {
        for text in [
            "Ich hoffe, dass du kommen sollt.",
            "Er sagte, dass wir müde bist.",
            "Sie weiß, dass ich zu spät bist.",
            "Er fragt, ob du das Buch habt.",
            "Das gilt, weil du zu jung bin.",
            "Man merkt es, sobald wir bereit bist.",
        ] {
            assert_eq!(lint_count(text), 1, "{text}");
        }
    }

    /// `es` is admitted behind a conjunction and nowhere else: the placeholder
    /// that made it unusable in the front field needs a front field to stand
    /// in.
    #[test]
    fn the_placeholder_is_a_real_pronoun_in_a_subordinate_clause() {
        assert_eq!(lint_count("Es ist nicht so wie es sein sollt."), 1);
        assert_eq!(lint_count("Es ist gut, weil es so gewesen wart."), 1);
        assert_eq!(lint_count("Ich glaube, dass es genug seid."), 1);
    }

    /// …and its number still comes from the predicate, so only the person is
    /// checked. *weil es meine Freunde sind* is correct German.
    #[test]
    fn the_placeholder_takes_its_number_from_the_predicate() {
        for text in [
            "Es ist so, weil es meine Freunde sind.",
            "Ich weiß, dass es viele Möglichkeiten gibt.",
            "Das stimmt, weil es zwei Gründe waren.",
            "Er sagt, dass es fünf Klassen sind.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// A correct verb-final clause stays quiet whatever sits between the
    /// subject and the verb.
    #[test]
    fn a_correct_verb_final_clause_is_left_alone() {
        for text in [
            "Es ist nicht so, wie es sein sollte.",
            "Er meint, dass du kommen kannst.",
            "Sie sagt, dass wir gegangen sind.",
            "Ich weiß, dass er das Buch gelesen hat.",
            "Das geht, solange du vorsichtig bist.",
            "Er wartet, bevor ich die Tür öffne.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// `-en` clause-finally is the infinitive as often as it is a finite verb,
    /// and an adjective besides. Every one of these ends on one.
    #[test]
    fn an_en_ending_is_never_trusted_at_the_end_of_a_clause() {
        for text in [
            "Er wird gereizt, damit er nicht mehr richtig urteilen kann.",
            "Gabriel singt davon, dass er versucht zu schlafen.",
            "Das ist so, damit diese erkennen und wie er handeln können.",
            "Ich bin kein Idealist, da ich nicht so weit gehe zu behaupten.",
            "Es ist bekannt, dass man das Wissen erarbeiten muss.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// An attributive adjective can end a scan when a coordinator cuts the
    /// clause short. It is not the verb.
    #[test]
    fn an_adjective_is_not_the_verb_of_the_clause() {
        for text in [
            "Es hängt davon ab, ob wir eine positive oder negative Wirkung erwarten.",
            "Da er jede Theorie in einem konkreten, empirischen Zusammenhang sieht, gilt das.",
            "Man nimmt an, dass er dem frühen 9. Jahrhundert entstammt.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// A coordinator starts a clause that has a subject of its own, so the
    /// word at the end of the sentence is not this subject's verb.
    #[test]
    fn a_coordinated_clause_belongs_to_its_own_subject() {
        for text in [
            "Ich sage, dass ich keine Beweise hätte und der Dieb es nicht zurückgibt.",
            "Es gilt, sobald wir einen anderen sehen oder uns sein Leiden geschildert wird.",
            "Er weiß, dass wir bereit sind und du später kommst.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// A question word is spelled like a conjunction, but puts the verb where
    /// the check expects the subject, so it never matches.
    #[test]
    fn a_question_is_not_a_subordinate_clause() {
        for text in [
            "Wie geht es dir?",
            "Wo bist du gewesen?",
            "Er ist größer wie ich.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// The clause ends at its own punctuation; a word from the next one is not
    /// reached.
    #[test]
    fn the_scan_stops_at_the_end_of_the_clause() {
        assert_eq!(lint_count("Weil du das machst, geht es."), 0);
        assert_eq!(lint_count("Obwohl wir müde sind, bleibt er wach."), 0);
        assert_eq!(lint_count("Wenn er kommt, sind wir bereit."), 0);
    }
}
