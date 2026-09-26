//! Inflectional features shared by the non-English language modules.
//!
//! English carries no morphology of this kind, so none of it belongs in
//! [`crate::dict_word_metadata`]. That module holds a single feature-gated
//! `morphology` field; everything that gives it meaning — the feature enums, the
//! merge behaviour and the accessors the linters call — lives here, behind the
//! `language-module` feature.
//!
//! These features are deliberately not German-specific. Case and gender are
//! needed just as much by the Slavic language modules, so this is shared
//! multilingual infrastructure rather than something under `german/`.

use is_macro::Is;
use serde::{Deserialize, Serialize};

use crate::dict_word_metadata::DictWordMetadata;

/// Grammatical case.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Is, Hash)]
pub enum Case {
    Nominative,
    Accusative,
    Dative,
    Genitive,
}

/// Grammatical number.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Is, Hash)]
pub enum Number {
    Singular,
    Plural,
}

/// Grammatical gender.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Is, Hash)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

/// Grammatical person, for the agreement between a subject and its verb.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Is, Hash)]
pub enum Person {
    First,
    Second,
    Third,
}

/// Verb mood.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Is, Hash)]
pub enum Mood {
    Indicative,
    Imperative,
    SubjunctiveI,
    SubjunctiveII,
}

bitflags::bitflags! {
    /// The cases a form can be read as.
    ///
    /// A set rather than a single value because inflected forms are routinely
    /// ambiguous, and in German pervasively so: a feminine noun carries no case
    /// marking in the singular at all, which makes *Frau* nominative,
    /// accusative, dative and genitive at once.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
    pub struct CaseSet: u8 {
        const NOMINATIVE = 1 << 0;
        const ACCUSATIVE = 1 << 1;
        const DATIVE     = 1 << 2;
        const GENITIVE   = 1 << 3;
    }

    /// The genders a form can be read as. See [`CaseSet`].
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
    pub struct GenderSet: u8 {
        const MASCULINE = 1 << 0;
        const FEMININE  = 1 << 1;
        const NEUTER    = 1 << 2;
    }

    /// The numbers a form can be read as. See [`CaseSet`].
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
    pub struct NumberSet: u8 {
        const SINGULAR = 1 << 0;
        const PLURAL   = 1 << 1;
    }

    /// The persons a form can be read as. See [`CaseSet`].
    ///
    /// German needs the set as badly here as anywhere: the present `-t` ending
    /// is third person singular and second person plural at once (*er lernt*,
    /// *ihr lernt*), and `-en` is first person plural, third person plural and
    /// the infinitive.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
    pub struct PersonSet: u8 {
        const FIRST  = 1 << 0;
        const SECOND = 1 << 1;
        const THIRD  = 1 << 2;
    }
}

/// Generates each axis's conversions, single-value view, agreement helpers and
/// the deserializer that accepts the annotation shapes.
macro_rules! feature_set {
    ($set:ty, $unit:ty, $de:ident, $( $variant:ident => $flag:ident ),+ $(,)?) => {
        impl From<$unit> for $set {
            fn from(value: $unit) -> Self {
                match value {
                    $( <$unit>::$variant => <$set>::$flag, )+
                }
            }
        }

        impl $set {
            /// The single feature this set allows, or `None` when it is unknown
            /// or ambiguous.
            ///
            /// Callers that decide something from one reading want this;
            /// callers that check agreement want [`Self::agrees_with`], which
            /// does not force a choice.
            pub fn unique(self) -> Option<$unit> {
                $( if self == <$set>::$flag { return Some(<$unit>::$variant); } )+
                None
            }

            /// Nothing is known about this axis.
            ///
            /// Distinct from "no reading agrees": the dictionary simply does
            /// not say. 99.6% of German noun entries are in this state for
            /// gender, so treating it as a constraint would make the agreement
            /// linters fire on nearly every noun phrase.
            pub fn is_unknown(self) -> bool {
                self.is_empty()
            }

            /// Could these two forms describe the same thing?
            ///
            /// The intersection has to be non-empty, except that an unknown
            /// side constrains nothing and so agrees with anything. Erring
            /// towards `true` is what keeps a sparse dictionary from producing
            /// false positives.
            pub fn agrees_with(self, other: Self) -> bool {
                self.is_unknown() || other.is_unknown() || self.intersects(other)
            }
        }

        /// Accepts `"Masculine"` and `["Masculine", "Neuter"]` — the variant
        /// names of the unit enum rather than the flag names bitflags prints,
        /// so `annotations.json` stays readable.
        fn $de<'de, D>(deserializer: D) -> Result<$set, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de::{SeqAccess, Visitor};

            struct V;

            impl<'de> Visitor<'de> for V {
                type Value = $set;

                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str(concat!("a ", stringify!($unit), " name, or a list of them"))
                }

                fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<$set, E> {
                    let unit = <$unit>::deserialize(serde::de::value::StrDeserializer::new(v))?;
                    Ok(<$set>::from(unit))
                }

                fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<$set, A::Error> {
                    let mut out = <$set>::empty();
                    while let Some(one) = seq.next_element::<$unit>()? {
                        out |= <$set>::from(one);
                    }
                    Ok(out)
                }
            }

            deserializer.deserialize_any(V)
        }
    };
}

feature_set!(CaseSet, Case, de_case_set,
    Nominative => NOMINATIVE,
    Accusative => ACCUSATIVE,
    Dative => DATIVE,
    Genitive => GENITIVE,
);
feature_set!(GenderSet, Gender, de_gender_set,
    Masculine => MASCULINE,
    Feminine => FEMININE,
    Neuter => NEUTER,
);
feature_set!(NumberSet, Number, de_number_set,
    Singular => SINGULAR,
    Plural => PLURAL,
);
feature_set!(PersonSet, Person, de_person_set,
    First => FIRST,
    Second => SECOND,
    Third => THIRD,
);

/// The case/gender/number features carried by one part of speech.
///
/// Kept per-POS rather than flattened onto [`Morphology`] because a single
/// headword is routinely several parts of speech at once — the German article
/// `der` is simultaneously a determiner and (in the dictionary as it stands) a
/// noun. Flattening would let a noun's gender leak into the determiner reading
/// and silently defeat the agreement linters, which compare the two.
///
/// Each axis is a set, and an empty set means the dictionary says nothing about
/// it. See [`CaseSet`] for why a single value will not do.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct Agreement {
    #[serde(
        default,
        deserialize_with = "de_case_set",
        skip_serializing_if = "CaseSet::is_empty"
    )]
    pub case: CaseSet,
    #[serde(
        default,
        deserialize_with = "de_gender_set",
        skip_serializing_if = "GenderSet::is_empty"
    )]
    pub gender: GenderSet,
    #[serde(
        default,
        deserialize_with = "de_number_set",
        skip_serializing_if = "NumberSet::is_empty"
    )]
    pub number: NumberSet,
    /// Only a verb and a subject carry this; every other part of speech leaves
    /// it empty, which the agreement helpers read as "says nothing".
    #[serde(
        default,
        deserialize_with = "de_person_set",
        skip_serializing_if = "PersonSet::is_empty"
    )]
    pub person: PersonSet,
}

impl Agreement {
    /// Produce a copy of `self` widened by the readings of `other`.
    ///
    /// This is a union, not a preference. Merging is what combines the repeated
    /// headwords of a dictionary file and the several affixes that produced one
    /// form, and each of those is another reading the form genuinely has —
    /// keeping only the first would throw away exactly the ambiguity the
    /// agreement check needs to see.
    pub fn or(&self, other: &Self) -> Self {
        Self {
            case: self.case | other.case,
            gender: self.gender | other.gender,
            number: self.number | other.number,
            person: self.person | other.person,
        }
    }

    /// Whether every axis is compatible. See [`CaseSet::agrees_with`].
    pub fn agrees_with(&self, other: &Self) -> bool {
        self.case.agrees_with(other.case)
            && self.gender.agrees_with(other.gender)
            && self.number.agrees_with(other.number)
            && self.person.agrees_with(other.person)
    }

    /// The features both readings allow, per axis.
    ///
    /// Unlike [`Self::agrees_with`] this does not treat an unknown axis as
    /// permissive: it returns what is actually shared, so chaining it across a
    /// noun phrase narrows the possibilities the way LanguageTool's
    /// `retainAll` does.
    pub fn intersect(&self, other: &Self) -> Self {
        Self {
            case: self.case & other.case,
            gender: self.gender & other.gender,
            number: self.number & other.number,
            person: self.person & other.person,
        }
    }

    /// Nothing is known about any axis.
    pub fn is_unknown(&self) -> bool {
        self.case.is_empty()
            && self.gender.is_empty()
            && self.number.is_empty()
            && self.person.is_empty()
    }
}

/// Inflectional features attached to a dictionary entry.
///
/// Deserialized straight out of a language's `annotations.json`, for example:
///
/// ```json
/// "M": { "metadata": { "noun": {}, "morphology": { "noun": { "gender": "Masculine" } } } }
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct Morphology {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noun: Option<Agreement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pronoun: Option<Agreement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub determiner: Option<Agreement>,
    /// A finite verb's person and number. The conjugation affixes each build
    /// exactly one combination, so this comes off the affix rather than the
    /// entry — see `german/annotations.json`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verb: Option<Agreement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mood: Option<Mood>,
    /// A word of foreign origin that appears lower case in running text
    /// (`de facto`, `Homo sapiens`). Not a misspelling and not a miscapitalized
    /// noun, so the capitalization linters leave it alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_foreign: Option<bool>,
}

impl Morphology {
    /// Produce a copy of `self` with the known properties of `other` set.
    ///
    /// Mirrors the `merge!` macro in [`DictWordMetadata::merge`], which is what
    /// combines the flags of repeated headwords in a dictionary file.
    pub fn or(&self, other: &Self) -> Self {
        fn or_agreement(a: Option<Agreement>, b: Option<Agreement>) -> Option<Agreement> {
            match (a, b) {
                (Some(a), Some(b)) => Some(a.or(&b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            }
        }

        Self {
            noun: or_agreement(self.noun, other.noun),
            pronoun: or_agreement(self.pronoun, other.pronoun),
            determiner: or_agreement(self.determiner, other.determiner),
            verb: or_agreement(self.verb, other.verb),
            mood: self.mood.or(other.mood),
            is_foreign: self.is_foreign.or(other.is_foreign),
        }
    }
}

/// Morphological queries over [`DictWordMetadata`].
///
/// These live on an extension trait rather than as inherent methods so that
/// `dict_word_metadata.rs` stays free of language-specific concepts. Import the
/// trait to use them:
///
/// ```ignore
/// use crate::language::morphology::MorphologyExt;
/// let gender = metadata.get_noun_gender();
/// ```
pub trait MorphologyExt {
    /// The raw feature bundle, if the entry has one.
    fn morphology(&self) -> Option<&Morphology>;

    /// The noun's agreement features, empty when the entry has none.
    fn noun_agreement(&self) -> Agreement {
        self.morphology().and_then(|m| m.noun).unwrap_or_default()
    }

    /// The pronoun's agreement features, empty when the entry has none.
    fn pronoun_agreement(&self) -> Agreement {
        self.morphology()
            .and_then(|m| m.pronoun)
            .unwrap_or_default()
    }

    /// The determiner's agreement features, empty when the entry has none.
    fn determiner_agreement(&self) -> Agreement {
        self.morphology()
            .and_then(|m| m.determiner)
            .unwrap_or_default()
    }

    // The `get_*` accessors below answer "which single feature is this?" and so
    // return `None` for an ambiguous form as well as for an unknown one. Use
    // the `*_agreement` accessors above when checking agreement, where the two
    // cases must be told apart.

    fn get_noun_case(&self) -> Option<Case> {
        self.noun_agreement().case.unique()
    }

    fn get_noun_gender(&self) -> Option<Gender> {
        self.noun_agreement().gender.unique()
    }

    fn get_noun_number(&self) -> Option<Number> {
        self.noun_agreement().number.unique()
    }

    fn get_pronoun_case(&self) -> Option<Case> {
        self.pronoun_agreement().case.unique()
    }

    fn get_pronoun_gender(&self) -> Option<Gender> {
        self.pronoun_agreement().gender.unique()
    }

    fn get_pronoun_number(&self) -> Option<Number> {
        self.pronoun_agreement().number.unique()
    }

    fn get_determiner_case(&self) -> Option<Case> {
        self.determiner_agreement().case.unique()
    }

    fn get_determiner_gender(&self) -> Option<Gender> {
        self.determiner_agreement().gender.unique()
    }

    fn get_verb_mood(&self) -> Option<Mood> {
        self.morphology()?.mood
    }

    /// Whether the entry is a lower-case foreign term (see [`Morphology::is_foreign`]).
    fn is_foreign_term(&self) -> bool {
        self.morphology()
            .and_then(|m| m.is_foreign)
            .unwrap_or(false)
    }

    /// Whether the entry carries any noun agreement features at all.
    fn has_noun_agreement(&self) -> bool {
        !self.noun_agreement().is_unknown()
    }
}

impl MorphologyExt for DictWordMetadata {
    fn morphology(&self) -> Option<&Morphology> {
        self.morphology.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noun_gender(gender: Gender) -> Morphology {
        Morphology {
            noun: Some(Agreement {
                gender: gender.into(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn default_morphology_is_all_none() {
        let m = Morphology::default();
        assert!(m.noun.is_none());
        assert!(m.pronoun.is_none());
        assert!(m.determiner.is_none());
        assert!(m.mood.is_none());
        assert!(m.is_foreign.is_none());
    }

    #[test]
    fn or_fills_missing_features() {
        let masculine = noun_gender(Gender::Masculine);
        let plural = Morphology {
            noun: Some(Agreement {
                number: Number::Plural.into(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let merged = masculine.or(&plural);
        let noun = merged.noun.unwrap();
        assert_eq!(noun.gender, GenderSet::MASCULINE);
        assert_eq!(noun.number, NumberSet::PLURAL);
    }

    #[test]
    /// Merging is a union, not a preference. Two dictionary lines for one
    /// headword are two readings the form really has, and dropping either is
    /// what made case unrepresentable before.
    fn or_unions_readings() {
        let merged = noun_gender(Gender::Masculine).or(&noun_gender(Gender::Feminine));
        let gender = merged.noun.unwrap().gender;
        assert_eq!(gender, GenderSet::MASCULINE | GenderSet::FEMININE);
        assert_eq!(
            gender.unique(),
            None,
            "an ambiguous set has no single reading"
        );
    }

    /// German feminine nouns carry no case marking in the singular, so one form
    /// really is all four cases. This is the representation that the previous
    /// `Option<Case>` could not express.
    #[test]
    fn a_form_can_hold_every_case_at_once() {
        let frau = Agreement {
            case: CaseSet::all(),
            gender: Gender::Feminine.into(),
            number: Number::Singular.into(),
            ..Agreement::default()
        };

        assert!(frau.case.contains(CaseSet::DATIVE));
        assert!(frau.case.contains(CaseSet::GENITIVE));
        assert_eq!(frau.case.unique(), None);
        assert_eq!(frau.gender.unique(), Some(Gender::Feminine));
    }

    /// The check the agreement linters are built on: *dem* (dative) and a form
    /// that can be dative agree; *des* (genitive only) and a dative-only form
    /// do not.
    #[test]
    fn agreement_is_a_non_empty_intersection() {
        let dative = Agreement {
            case: CaseSet::DATIVE,
            ..Default::default()
        };
        let genitive = Agreement {
            case: CaseSet::GENITIVE,
            ..Default::default()
        };
        let either = Agreement {
            case: CaseSet::DATIVE | CaseSet::GENITIVE,
            ..Default::default()
        };

        assert!(dative.agrees_with(&either));
        assert!(genitive.agrees_with(&either));
        assert!(!dative.agrees_with(&genitive));
    }

    /// 99.6% of German noun entries carry no gender. An unknown axis has to
    /// constrain nothing, or the linters would fire on nearly every noun
    /// phrase.
    #[test]
    fn an_unknown_axis_agrees_with_anything() {
        let unknown = Agreement::default();
        let feminine = Agreement {
            gender: Gender::Feminine.into(),
            ..Default::default()
        };

        assert!(unknown.agrees_with(&feminine));
        assert!(feminine.agrees_with(&unknown));
        assert!(unknown.is_unknown());
    }

    /// Chaining `intersect` down a noun phrase narrows the readings, the way
    /// LanguageTool's `retainAll` does. Unlike `agrees_with` it does not treat
    /// an unknown side as permissive, so callers must check `is_unknown` first.
    #[test]
    fn intersect_narrows_the_readings() {
        let article = Agreement {
            case: CaseSet::NOMINATIVE | CaseSet::ACCUSATIVE,
            gender: GenderSet::MASCULINE | GenderSet::NEUTER,
            ..Default::default()
        };
        let noun = Agreement {
            case: CaseSet::ACCUSATIVE | CaseSet::DATIVE,
            gender: GenderSet::NEUTER,
            ..Default::default()
        };

        let narrowed = article.intersect(&noun);
        assert_eq!(narrowed.case, CaseSet::ACCUSATIVE);
        assert_eq!(narrowed.gender, GenderSet::NEUTER);
    }

    /// The annotation shape stays as it was, and a list is now accepted too.
    #[test]
    fn deserializes_a_single_feature_and_a_list() {
        let one: Agreement = serde_json::from_str(r#"{"gender":"Masculine"}"#).unwrap();
        assert_eq!(one.gender, GenderSet::MASCULINE);

        let many: Agreement =
            serde_json::from_str(r#"{"case":["Dative","Genitive"],"gender":"Feminine"}"#).unwrap();
        assert_eq!(many.case, CaseSet::DATIVE | CaseSet::GENITIVE);
        assert_eq!(many.gender, GenderSet::FEMININE);
    }

    /// An empty axis must not be written out, so existing dictionary artifacts
    /// do not grow a key each.
    #[test]
    fn empty_axes_are_not_serialized() {
        let json = serde_json::to_string(&Agreement {
            gender: Gender::Neuter.into(),
            ..Default::default()
        })
        .unwrap();

        assert!(!json.contains("case"), "{json}");
        assert!(!json.contains("number"), "{json}");
        assert!(json.contains("gender"), "{json}");
    }

    /// A headword that is both a determiner and a noun must not have its noun
    /// gender read back as determiner gender; the agreement linters compare the
    /// two and would never fire again.
    #[test]
    fn noun_features_do_not_leak_into_determiner() {
        let meta = DictWordMetadata {
            morphology: Some(noun_gender(Gender::Masculine)),
            ..Default::default()
        };

        assert_eq!(meta.get_noun_gender(), Some(Gender::Masculine));
        assert_eq!(meta.get_determiner_gender(), None);
        assert_eq!(meta.get_pronoun_gender(), None);
    }

    #[test]
    fn accessors_are_none_without_morphology() {
        let meta = DictWordMetadata::default();
        assert_eq!(meta.get_noun_gender(), None);
        assert_eq!(meta.get_noun_number(), None);
        assert_eq!(meta.get_verb_mood(), None);
        assert!(!meta.is_foreign_term());
        assert!(!meta.has_noun_agreement());
    }

    #[test]
    fn absent_morphology_round_trips_without_null_keys() {
        let meta = DictWordMetadata::default();
        let json = serde_json::to_string(&meta).unwrap();
        assert!(
            !json.contains("morphology"),
            "an entry without morphology must not serialize the key: {json}"
        );
    }

    #[test]
    fn deserializes_from_annotation_shape() {
        let meta: DictWordMetadata =
            serde_json::from_str(r#"{"noun":{},"morphology":{"noun":{"gender":"Masculine"}}}"#)
                .unwrap();
        assert_eq!(meta.get_noun_gender(), Some(Gender::Masculine));
    }

    #[test]
    fn merge_unions_morphology_across_dictionary_lines() {
        let mut a = DictWordMetadata {
            morphology: Some(noun_gender(Gender::Neuter)),
            ..Default::default()
        };

        let b = DictWordMetadata {
            morphology: Some(Morphology {
                mood: Some(Mood::Imperative),
                ..Default::default()
            }),
            ..Default::default()
        };

        a.merge(&b);
        assert_eq!(a.get_noun_gender(), Some(Gender::Neuter));
        assert_eq!(a.get_verb_mood(), Some(Mood::Imperative));
    }

    #[test]
    fn merge_keeps_morphology_from_either_side() {
        let mut none_side = DictWordMetadata::default();
        let some_side = DictWordMetadata {
            morphology: Some(noun_gender(Gender::Feminine)),
            ..Default::default()
        };

        none_side.merge(&some_side);
        assert_eq!(none_side.get_noun_gender(), Some(Gender::Feminine));
    }

    /// The German present `-t` ending is third person singular and second
    /// person plural at once, which is the reason this axis is a set.
    #[test]
    fn one_ending_can_hold_two_persons() {
        let lernt = Agreement {
            number: NumberSet::SINGULAR | NumberSet::PLURAL,
            person: PersonSet::SECOND | PersonSet::THIRD,
            ..Agreement::default()
        };

        assert_eq!(lernt.person.unique(), None);
        assert!(lernt.person.contains(PersonSet::THIRD));
        assert!(!lernt.person.contains(PersonSet::FIRST));
    }

    /// A subject and a verb agree when their persons intersect.
    #[test]
    fn a_subject_and_a_verb_have_to_share_a_person() {
        let du = Agreement {
            number: Number::Singular.into(),
            person: Person::Second.into(),
            ..Agreement::default()
        };
        let lernst = Agreement {
            number: Number::Singular.into(),
            person: Person::Second.into(),
            ..Agreement::default()
        };
        let lernt = Agreement {
            number: NumberSet::SINGULAR | NumberSet::PLURAL,
            person: PersonSet::SECOND | PersonSet::THIRD,
            ..Agreement::default()
        };
        let lerne = Agreement {
            number: Number::Singular.into(),
            person: Person::First.into(),
            ..Agreement::default()
        };

        assert!(du.agrees_with(&lernst));
        assert!(
            du.agrees_with(&lernt),
            "*du lernt* is wrong but not by person"
        );
        assert!(!du.agrees_with(&lerne), "*du lerne* shares no person");
    }

    /// An empty person set says nothing, so it agrees with everything. Every
    /// noun, determiner and adjective in the dictionary is in that state.
    #[test]
    fn an_unknown_person_constrains_nothing() {
        let silent = Agreement::default();
        let ich = Agreement {
            person: Person::First.into(),
            ..Agreement::default()
        };

        assert!(silent.agrees_with(&ich));
        assert!(ich.agrees_with(&silent));
        assert!(silent.is_unknown());
    }

    /// Intersecting does not treat the unknown side as permissive, so chaining
    /// it across a phrase narrows rather than widens.
    #[test]
    fn intersecting_persons_narrows() {
        let both = Agreement {
            person: PersonSet::SECOND | PersonSet::THIRD,
            ..Agreement::default()
        };
        let third = Agreement {
            person: Person::Third.into(),
            ..Agreement::default()
        };

        assert_eq!(both.intersect(&third).person, PersonSet::THIRD);
    }
}
