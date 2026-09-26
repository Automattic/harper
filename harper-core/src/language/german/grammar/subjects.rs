//! Personal pronouns and the finite verbs the affix rules cannot build.
//!
//! Subject–verb agreement relates two things the dictionary describes badly.
//!
//! The **subject pronouns** are a closed class of eight forms and their
//! readings are joint: *sie* is third person singular and third person plural
//! at once, and the polite *Sie* on top. Two independent axes would also admit
//! "third person, either number" for *er*, which is wrong. The determiner table
//! next door solves the same problem the same way.
//!
//! The **auxiliaries and modals** are irregular, so they sit in
//! `dictionary.dict` as whole words rather than being built by a conjugation
//! affix — and it is the affix that carries person and number. *ist*, *hat*,
//! *sind*, *kann* therefore have no features at all and never will, short of
//! spending flag letters on them. There are about sixty of them and they are
//! the most frequent verbs in the language, so they are written out here.
//!
//! Everything regular is left to the affixes; see `annotations.json`.

use hashbrown::HashMap;
use std::sync::LazyLock;

use crate::language::morphology::{Agreement, NumberSet, PersonSet};

/// The person and number of one form, as a pair of sets.
///
/// Unlike a determiner reading this does not need a list of fully specified
/// alternatives: person and number happen to be independent for every German
/// subject pronoun. *sie* is third person, singular or plural, and there is no
/// forbidden combination to exclude.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Features {
    pub person: PersonSet,
    pub number: NumberSet,
}

impl Features {
    pub const fn new(person: PersonSet, number: NumberSet) -> Self {
        Self { person, number }
    }

    pub fn agreement(&self) -> Agreement {
        Agreement {
            person: self.person,
            number: self.number,
            ..Agreement::default()
        }
    }

    /// Could a subject with these features take a verb with those?
    ///
    /// Both axes have to intersect. An empty set on either side is unknown and
    /// agrees with anything, which is [`Agreement::agrees_with`]'s rule and the
    /// reason a sparse dictionary does not produce false positives here.
    pub fn agrees_with(&self, other: &Features) -> bool {
        self.person.agrees_with(other.person) && self.number.agrees_with(other.number)
    }
}

const SG: NumberSet = NumberSet::SINGULAR;
const PL: NumberSet = NumberSet::PLURAL;
const P1: PersonSet = PersonSet::FIRST;
const P2: PersonSet = PersonSet::SECOND;
const P3: PersonSet = PersonSet::THIRD;

/// The nominative personal pronouns, and nothing else.
///
/// Oblique forms are deliberately absent: *ihn*, *ihm*, *mir*, *dich* cannot be
/// subjects, and reading one as a subject is how a rule like this invents
/// errors.
///
/// Two nominative forms are left out as well, both measured:
///
/// * **`es`**, because German uses it as a placeholder in the front field while
///   the real subject follows the verb: *Es werden fünf Klassen gebildet*, *Es
///   existieren zahlreiche Ansätze*. The verb agrees with the noun behind it,
///   not with *es*, and nothing short of finding that noun can tell the
///   placeholder from the pronoun. On a corpus of edited prose this one form
///   accounted for two hundred false reports.
/// * **`ihr`**, which is a possessive (*ihr Auto*) and a dative (*ich gebe ihr
///   Geld*) far more often than it is the second person plural.
const PRONOUNS: &[(&str, Features)] = &[
    ("ich", Features::new(P1, SG)),
    ("du", Features::new(P2, SG)),
    ("er", Features::new(P3, SG)),
    ("man", Features::new(P3, SG)),
    // Third person, either number: *sie schläft* and *sie schlafen*. The polite
    // *Sie* is the plural of this same form, so no separate entry.
    ("sie", Features::new(P3, SG.union(PL))),
    ("wir", Features::new(P1, PL)),
];

/// Finite forms of the verbs no affix rule builds.
///
/// `sein`, `haben` and `werden` carry most of the German verb system between
/// them, and the six modals are not far behind. Every one of them is irregular
/// enough that `dictionary.dict` holds the forms as separate words, which means
/// no affix and so no person.
///
/// Preterite forms are here too, and the subjunctive where it is spelled
/// differently from the indicative (*wäre*, *hätte*, *könnte*), because those
/// are the forms reported speech uses.
const FINITE_VERBS: &[(&str, Features)] = &[
    // sein
    ("bin", Features::new(P1, SG)),
    ("bist", Features::new(P2, SG)),
    ("ist", Features::new(P3, SG)),
    ("sind", Features::new(P1.union(P3), PL)),
    ("seid", Features::new(P2, PL)),
    ("war", Features::new(P1.union(P3), SG)),
    ("warst", Features::new(P2, SG)),
    ("waren", Features::new(P1.union(P3), PL)),
    ("wart", Features::new(P2, PL)),
    ("wäre", Features::new(P1.union(P3), SG)),
    ("wärst", Features::new(P2, SG)),
    ("wären", Features::new(P1.union(P3), PL)),
    // haben
    ("habe", Features::new(P1.union(P3), SG)),
    ("hast", Features::new(P2, SG)),
    ("hat", Features::new(P3, SG)),
    ("haben", Features::new(P1.union(P3), PL)),
    ("habt", Features::new(P2, PL)),
    ("hatte", Features::new(P1.union(P3), SG)),
    ("hattest", Features::new(P2, SG)),
    ("hatten", Features::new(P1.union(P3), PL)),
    ("hattet", Features::new(P2, PL)),
    ("hätte", Features::new(P1.union(P3), SG)),
    ("hätten", Features::new(P1.union(P3), PL)),
    // werden
    ("werde", Features::new(P1.union(P3), SG)),
    ("wirst", Features::new(P2, SG)),
    ("wird", Features::new(P3, SG)),
    ("werden", Features::new(P1.union(P3), PL)),
    ("werdet", Features::new(P2, PL)),
    ("wurde", Features::new(P1.union(P3), SG)),
    ("wurdest", Features::new(P2, SG)),
    ("wurden", Features::new(P1.union(P3), PL)),
    ("würde", Features::new(P1.union(P3), SG)),
    ("würden", Features::new(P1.union(P3), PL)),
    // können
    ("kann", Features::new(P1.union(P3), SG)),
    ("kannst", Features::new(P2, SG)),
    ("können", Features::new(P1.union(P3), PL)),
    ("könnt", Features::new(P2, PL)),
    ("konnte", Features::new(P1.union(P3), SG)),
    ("konnten", Features::new(P1.union(P3), PL)),
    ("könnte", Features::new(P1.union(P3), SG)),
    ("könnten", Features::new(P1.union(P3), PL)),
    // müssen
    ("muss", Features::new(P1.union(P3), SG)),
    ("musst", Features::new(P2, SG)),
    ("müssen", Features::new(P1.union(P3), PL)),
    ("müsst", Features::new(P2, PL)),
    ("musste", Features::new(P1.union(P3), SG)),
    ("mussten", Features::new(P1.union(P3), PL)),
    // wollen
    ("will", Features::new(P1.union(P3), SG)),
    ("willst", Features::new(P2, SG)),
    ("wollen", Features::new(P1.union(P3), PL)),
    ("wollt", Features::new(P2, PL)),
    ("wollte", Features::new(P1.union(P3), SG)),
    ("wollten", Features::new(P1.union(P3), PL)),
    // sollen
    ("soll", Features::new(P1.union(P3), SG)),
    ("sollst", Features::new(P2, SG)),
    ("sollen", Features::new(P1.union(P3), PL)),
    ("sollt", Features::new(P2, PL)),
    ("sollte", Features::new(P1.union(P3), SG)),
    ("sollten", Features::new(P1.union(P3), PL)),
    // dürfen
    ("darf", Features::new(P1.union(P3), SG)),
    ("darfst", Features::new(P2, SG)),
    ("dürfen", Features::new(P1.union(P3), PL)),
    ("dürft", Features::new(P2, PL)),
    ("durfte", Features::new(P1.union(P3), SG)),
    ("durften", Features::new(P1.union(P3), PL)),
    // mögen
    ("mag", Features::new(P1.union(P3), SG)),
    ("magst", Features::new(P2, SG)),
    ("mögen", Features::new(P1.union(P3), PL)),
    ("mögt", Features::new(P2, PL)),
    ("möchte", Features::new(P1.union(P3), SG)),
    ("möchten", Features::new(P1.union(P3), PL)),
    // wissen
    ("weiß", Features::new(P1.union(P3), SG)),
    ("weißt", Features::new(P2, SG)),
    ("wissen", Features::new(P1.union(P3), PL)),
    ("wisst", Features::new(P2, PL)),
    ("wusste", Features::new(P1.union(P3), SG)),
    ("wussten", Features::new(P1.union(P3), PL)),
];

static PRONOUN_MAP: LazyLock<HashMap<&'static str, Features>> =
    LazyLock::new(|| PRONOUNS.iter().copied().collect());

static FINITE_VERB_MAP: LazyLock<HashMap<&'static str, Features>> =
    LazyLock::new(|| FINITE_VERBS.iter().copied().collect());

/// The features of `word` read as a subject pronoun, if it is one.
pub fn subject_pronoun(word: &str) -> Option<Features> {
    PRONOUN_MAP.get(word.to_lowercase().as_str()).copied()
}

/// The features of `word` read as a finite verb the affixes do not build.
pub fn irregular_finite_verb(word: &str) -> Option<Features> {
    FINITE_VERB_MAP.get(word.to_lowercase().as_str()).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pronoun_agrees_with_its_own_verb() {
        let du = subject_pronoun("du").unwrap();
        assert!(du.agrees_with(&irregular_finite_verb("bist").unwrap()));
        assert!(du.agrees_with(&irregular_finite_verb("hast").unwrap()));
        assert!(!du.agrees_with(&irregular_finite_verb("ist").unwrap()));
        assert!(!du.agrees_with(&irregular_finite_verb("sind").unwrap()));
    }

    /// *sie* is third person in both numbers, so both verbs fit it.
    #[test]
    fn sie_is_singular_and_plural() {
        let sie = subject_pronoun("Sie").unwrap();
        assert!(sie.agrees_with(&irregular_finite_verb("ist").unwrap()));
        assert!(sie.agrees_with(&irregular_finite_verb("sind").unwrap()));
        assert!(!sie.agrees_with(&irregular_finite_verb("bin").unwrap()));
    }

    #[test]
    fn oblique_pronouns_are_not_subjects() {
        for word in ["ihn", "ihm", "mir", "mich", "dich", "uns", "euch", "ihnen"] {
            assert!(
                subject_pronoun(word).is_none(),
                "{word} cannot be a subject"
            );
        }
    }

    #[test]
    fn every_entry_is_lower_case_and_unique() {
        for (word, _) in PRONOUNS.iter().chain(FINITE_VERBS) {
            assert_eq!(*word, word.to_lowercase(), "{word} must be lower case");
        }
        assert_eq!(PRONOUN_MAP.len(), PRONOUNS.len(), "a pronoun is repeated");
        assert_eq!(
            FINITE_VERB_MAP.len(),
            FINITE_VERBS.len(),
            "a verb form is repeated"
        );
    }

    /// No form may be both, or the rule would compare a word with itself.
    #[test]
    fn no_form_is_both_a_pronoun_and_a_verb() {
        for (word, _) in PRONOUNS {
            assert!(
                irregular_finite_verb(word).is_none(),
                "{word} is in both tables"
            );
        }
    }
}
