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

/// Verb mood.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Is, Hash)]
pub enum Mood {
    Indicative,
    Imperative,
    SubjunctiveI,
    SubjunctiveII,
}

/// The case/gender/number features carried by one part of speech.
///
/// Kept per-POS rather than flattened onto [`Morphology`] because a single
/// headword is routinely several parts of speech at once — the German article
/// `der` is simultaneously a determiner and (in the dictionary as it stands) a
/// noun. Flattening would let a noun's gender leak into the determiner reading
/// and silently defeat the agreement linters, which compare the two.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct Agreement {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case: Option<Case>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gender: Option<Gender>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<Number>,
}

impl Agreement {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        Self {
            case: self.case.or(other.case),
            gender: self.gender.or(other.gender),
            number: self.number.or(other.number),
        }
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

    fn get_noun_case(&self) -> Option<Case> {
        self.morphology()?.noun?.case
    }

    fn get_noun_gender(&self) -> Option<Gender> {
        self.morphology()?.noun?.gender
    }

    fn get_noun_number(&self) -> Option<Number> {
        self.morphology()?.noun?.number
    }

    fn get_pronoun_case(&self) -> Option<Case> {
        self.morphology()?.pronoun?.case
    }

    fn get_pronoun_gender(&self) -> Option<Gender> {
        self.morphology()?.pronoun?.gender
    }

    fn get_pronoun_number(&self) -> Option<Number> {
        self.morphology()?.pronoun?.number
    }

    fn get_determiner_case(&self) -> Option<Case> {
        self.morphology()?.determiner?.case
    }

    fn get_determiner_gender(&self) -> Option<Gender> {
        self.morphology()?.determiner?.gender
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
        self.get_noun_gender().is_some() || self.get_noun_number().is_some()
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
                gender: Some(gender),
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
                number: Some(Number::Plural),
                ..Default::default()
            }),
            ..Default::default()
        };

        let merged = masculine.or(&plural);
        let noun = merged.noun.unwrap();
        assert_eq!(noun.gender, Some(Gender::Masculine));
        assert_eq!(noun.number, Some(Number::Plural));
    }

    #[test]
    fn or_prefers_self() {
        let merged = noun_gender(Gender::Masculine).or(&noun_gender(Gender::Feminine));
        assert_eq!(merged.noun.unwrap().gender, Some(Gender::Masculine));
    }

    /// A headword that is both a determiner and a noun must not have its noun
    /// gender read back as determiner gender; the agreement linters compare the
    /// two and would never fire again.
    #[test]
    fn noun_features_do_not_leak_into_determiner() {
        let mut meta = DictWordMetadata::default();
        meta.morphology = Some(noun_gender(Gender::Masculine));

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
        let mut a = DictWordMetadata::default();
        a.morphology = Some(noun_gender(Gender::Neuter));

        let mut b = DictWordMetadata::default();
        b.morphology = Some(Morphology {
            mood: Some(Mood::Imperative),
            ..Default::default()
        });

        a.merge(&b);
        assert_eq!(a.get_noun_gender(), Some(Gender::Neuter));
        assert_eq!(a.get_verb_mood(), Some(Mood::Imperative));
    }

    #[test]
    fn merge_keeps_morphology_from_either_side() {
        let mut none_side = DictWordMetadata::default();
        let mut some_side = DictWordMetadata::default();
        some_side.morphology = Some(noun_gender(Gender::Feminine));

        none_side.merge(&some_side);
        assert_eq!(none_side.get_noun_gender(), Some(Gender::Feminine));
    }
}
