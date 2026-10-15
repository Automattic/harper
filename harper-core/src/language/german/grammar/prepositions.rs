//! Which case each German preposition governs.
//!
//! Compiled from the standard grammatical descriptions (Duden, *Die Grammatik*,
//! §§ 906–922) rather than from another checker's table. The entries are plain
//! facts about the language, but the decisions around them are not, and three
//! of them are worth stating.
//!
//! **Two-way prepositions get both cases.** *an, auf, hinter, in, neben, über,
//! unter, vor, zwischen* take the accusative for a direction and the dative for
//! a location. Nothing in the noun phrase itself says which, so they are
//! recorded as `ACCUSATIVE | DATIVE` and can only catch a determiner that is
//! neither.
//!
//! **Some prepositions are also conjunctions.** In *während das Kind schlief*,
//! *während* introduces a clause and *das Kind* is its nominative subject — not
//! a genitive that has gone wrong. [`Preposition::also_a_conjunction`] marks
//! these, and the linter then reports only the dative, which no subject can be.
//!
//! **Two are left out entirely.** *bis* and *seit* are conjunctions often
//! enough (*bis der Zug kommt*, *seit das Kind hier ist*) that the same trick
//! does not save them: their own case, the accusative and the dative, is
//! exactly what a mistake would look like. They take a determiner directly so
//! rarely — *bis zum Ende*, *seit dem Unfall* — that dropping them costs
//! almost nothing.

use hashbrown::HashMap;
use std::sync::LazyLock;

use crate::language::morphology::CaseSet;

/// What a preposition requires of the noun phrase that follows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Preposition {
    /// The cases the preposition governs.
    pub cases: CaseSet,
    /// The word also introduces a subordinate clause, whose subject is
    /// nominative. See the module comment.
    pub also_a_conjunction: bool,
    /// The word also heads an infinitive clause — *um … zu*, *ohne … zu*,
    /// *anstatt … zu*. The noun phrase inside one belongs to the infinitive
    /// and takes its case, not the preposition's: in *um dem Leser eine
    /// spannende Handlung zu bieten* the dative is what *bieten* wants.
    pub also_an_infinitive_clause: bool,
    /// The word is also used after its noun phrase rather than before it —
    /// *seiner Ansicht nach*, *ihm zufolge*, *den Menschen gegenüber*. What
    /// follows such a postposition begins a new phrase and is not governed by
    /// it.
    pub also_a_postposition: bool,
}

const AKK: CaseSet = CaseSet::ACCUSATIVE;
const DAT: CaseSet = CaseSet::DATIVE;
const GEN: CaseSet = CaseSet::GENITIVE;

/// Governs the accusative.
const ACCUSATIVE: &[&str] = &[
    "durch", "für", "gegen", "kontra", "ohne", "per", "pro", "um", "versus", "via", "wider",
];

/// Governs the dative.
const DATIVE: &[&str] = &[
    "ab",
    "aus",
    "außer",
    "bei",
    "entgegen",
    "fern",
    "gegenüber",
    "gemäß",
    "getreu",
    "mit",
    "mitsamt",
    "nach",
    "nächst",
    "nebst",
    "samt",
    "von",
    "zu",
    "zuliebe",
    "zuwider",
];

/// Takes the accusative for a direction and the dative for a location.
const TWO_WAY: &[&str] = &[
    "an", "auf", "hinter", "in", "neben", "über", "unter", "vor", "zwischen",
];

/// Governs the genitive.
const GENITIVE: &[&str] = &[
    "abseits",
    "abzüglich",
    "angesichts",
    "anhand",
    "anlässlich",
    "anstelle",
    "aufgrund",
    "ausgangs",
    "ausweislich",
    "außerhalb",
    "behufs",
    "beidseits",
    "betreffs",
    "halber",
    "bezüglich",
    "diesseits",
    "eingangs",
    "eingedenk",
    "einschließlich",
    "fernab",
    "hinsichtlich",
    "infolge",
    "inmitten",
    "innerhalb",
    "innert",
    "jenseits",
    "kraft",
    "längs",
    "längsseits",
    "mangels",
    "mittels",
    "namens",
    "oberhalb",
    "rücksichtlich",
    "seitens",
    "seitlich",
    "trotz",
    "unbeschadet",
    "ungeachtet",
    "unterhalb",
    "unweit",
    "vermittels",
    "vermittelst",
    "vermöge",
    "vorbehaltlich",
    "wegen",
    "weitab",
    "zugunsten",
    "zuungunsten",
    "zuzüglich",
    "zwecks",
];

/// Prepositions whose government is genuinely shared between two cases, and the
/// ones that are also conjunctions.
/// Prepositions that need more than a case: a second standard case, or a
/// second syntactic role that the linter has to keep out of.
const SPECIAL: &[(&str, CaseSet, Role)] = &[
    // Both cases are standard; the dative is the usual one.
    ("dank", DAT.union(GEN), Role::PLAIN),
    ("laut", DAT.union(GEN), Role::PLAIN),
    ("binnen", DAT.union(GEN), Role::PLAIN),
    // A postposition with the accusative, a preposition with the dative, and
    // the genitive in careful writing: nothing left to rule out.
    ("entlang", AKK.union(DAT).union(GEN), Role::POSTPOSITION),
    // Genitive prepositions that also introduce a clause.
    ("während", GEN, Role::CONJUNCTION.with(Role::INFINITIVE)),
    ("statt", GEN, Role::CONJUNCTION.with(Role::INFINITIVE)),
    ("anstatt", GEN, Role::CONJUNCTION.with(Role::INFINITIVE)),
    // Heads of infinitive clauses.
    ("um", AKK, Role::INFINITIVE),
    ("ohne", AKK, Role::INFINITIVE),
    ("außer", DAT, Role::INFINITIVE),
    // Used after their noun phrase at least as often as before it.
    ("nach", DAT, Role::POSTPOSITION),
    ("zufolge", DAT.union(GEN), Role::POSTPOSITION),
    ("gegenüber", DAT, Role::POSTPOSITION),
    ("entgegen", DAT, Role::POSTPOSITION),
    ("zuliebe", DAT, Role::POSTPOSITION),
    ("zuwider", DAT, Role::POSTPOSITION),
    ("halber", GEN, Role::POSTPOSITION),
    ("gemäß", DAT, Role::POSTPOSITION),
    ("wegen", GEN, Role::POSTPOSITION),
    // *von dem Punkt aus*, *von Natur aus*: the second half of a frame whose
    // case belongs to *von*.
    ("aus", DAT, Role::POSTPOSITION),
];

/// The extra syntactic roles a preposition's spelling also has.
#[derive(Clone, Copy)]
struct Role(u8);

impl Role {
    const PLAIN: Role = Role(0);
    const CONJUNCTION: Role = Role(1);
    const INFINITIVE: Role = Role(2);
    const POSTPOSITION: Role = Role(4);

    const fn with(self, other: Role) -> Role {
        Role(self.0 | other.0)
    }

    const fn has(self, other: Role) -> bool {
        self.0 & other.0 != 0
    }
}

static PREPOSITIONS: LazyLock<HashMap<&'static str, Preposition>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    let mut insert = |word: &'static str, cases: CaseSet, role: Role| {
        map.insert(
            word,
            Preposition {
                cases,
                also_a_conjunction: role.has(Role::CONJUNCTION),
                also_an_infinitive_clause: role.has(Role::INFINITIVE),
                also_a_postposition: role.has(Role::POSTPOSITION),
            },
        );
    };

    for (words, cases) in [
        (ACCUSATIVE, AKK),
        (DATIVE, DAT),
        (TWO_WAY, AKK.union(DAT)),
        (GENITIVE, GEN),
    ] {
        for word in words {
            insert(word, cases, Role::PLAIN);
        }
    }

    // After the groups, so a word listed in both takes its special entry.
    for (word, cases, role) in SPECIAL {
        insert(word, *cases, *role);
    }

    map
});

/// What `word` requires of the noun phrase after it, or `None` if it is not a
/// preposition this table covers.
pub fn preposition_government(word: &str) -> Option<Preposition> {
    PREPOSITIONS.get(word.to_lowercase().as_str()).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cases(word: &str) -> CaseSet {
        preposition_government(word)
            .unwrap_or_else(|| panic!("{word} is not in the table"))
            .cases
    }

    #[test]
    fn the_four_groups_are_read_back() {
        assert_eq!(cases("für"), CaseSet::ACCUSATIVE);
        assert_eq!(cases("mit"), CaseSet::DATIVE);
        assert_eq!(cases("wegen"), CaseSet::GENITIVE);
        assert_eq!(cases("auf"), CaseSet::ACCUSATIVE | CaseSet::DATIVE);
    }

    #[test]
    fn a_sentence_initial_preposition_is_found() {
        assert_eq!(cases("Mit"), CaseSet::DATIVE);
    }

    #[test]
    fn prepositions_with_two_standard_cases_accept_both() {
        assert_eq!(cases("dank"), CaseSet::DATIVE | CaseSet::GENITIVE);
        assert_eq!(cases("laut"), CaseSet::DATIVE | CaseSet::GENITIVE);
    }

    #[test]
    fn conjunction_homographs_are_marked() {
        assert!(
            preposition_government("während")
                .unwrap()
                .also_a_conjunction
        );
        assert!(preposition_government("statt").unwrap().also_a_conjunction);
        assert!(!preposition_government("wegen").unwrap().also_a_conjunction);
    }

    /// See the module comment: these two are conjunctions too often, and the
    /// case they govern is the same one a mistake would produce.
    #[test]
    fn bis_and_seit_are_deliberately_absent() {
        assert!(preposition_government("bis").is_none());
        assert!(preposition_government("seit").is_none());
    }

    /// Every one of these is a noun, an adverb or a quantifier far more often
    /// than a preposition, and each produced a false positive on the prose
    /// corpus.
    #[test]
    fn noun_and_adverb_homographs_are_left_out() {
        for word in [
            "zeit",
            "bar",
            "je",
            "ausschließlich",
            "entsprechend",
            "inklusive",
            "exklusive",
        ] {
            assert!(preposition_government(word).is_none(), "{word}");
        }
    }

    #[test]
    fn the_second_roles_are_recorded() {
        assert!(
            preposition_government("um")
                .unwrap()
                .also_an_infinitive_clause
        );
        assert!(
            preposition_government("ohne")
                .unwrap()
                .also_an_infinitive_clause
        );
        assert!(preposition_government("nach").unwrap().also_a_postposition);
        assert!(
            preposition_government("zufolge")
                .unwrap()
                .also_a_postposition
        );
        assert!(!preposition_government("mit").unwrap().also_a_postposition);
    }

    #[test]
    fn ordinary_words_are_not_prepositions() {
        assert!(preposition_government("Haus").is_none());
        assert!(preposition_government("und").is_none());
    }
}
