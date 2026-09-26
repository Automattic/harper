//! The verb has to match the subject in person and number.

use std::sync::Arc;

use crate::language::german::grammar::subjects::{
    Features, irregular_finite_verb, subject_pronoun,
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
/// Only the **front field** is checked: the pronoun has to open its clause, and
/// the verb has to stand directly behind it. German is verb-second, so that is
/// the one position where the finite verb is guaranteed to be the next word.
/// Everywhere else it is not, and the corpus says so loudly — in *das er
/// vergessen hat* and *weil er gehen muss* the word behind the pronoun is a
/// participle or an infinitive and the finite verb is at the end of the clause.
/// Checking those pairs reported eight hundred times on edited prose and was
/// wrong every time.
///
/// A **noun-phrase subject** — *die Kinder spielt* — is not checked, and the
/// attempt is worth recording. Finding the head is easy enough, but the word
/// behind it is not reliably the verb: *die Gesellschaft bürgerlichen Rechts*
/// and *die Arten hohler Stängel* put an adjective there, and a relative clause
/// behind a comma (*…, welches Sittenwidrigkeit impliziert*) passes the
/// front-field test while being verb-final. That version reported 1229 times on
/// the same prose. It needs the noun-phrase chunker the capitalization rule
/// has, not another guard.
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
        // What the part of speech does here is rule things out. The article
        // `die` carries a verb reading, because the root `dien` is inflected as
        // if it were an infinitive — see `tests/verb_person_test.rs` — and the
        // `-st` affix turns `selb` into `selbst`. A determiner, a pronoun or an
        // adverb is not the finite verb of the clause.
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

                lints.push(Lint {
                    span: verb_token.span,
                    lint_kind: LintKind::Agreement,
                    suggestions: Vec::new(),
                    priority: 30,
                    message: format!(
                        "»{subject_text}« verlangt {}. »{verb_text}« steht in einer anderen Form.",
                        Self::wanted(&subject)
                    ),
                });
            }
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
}
