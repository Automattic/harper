use is_macro::Is;
use paste::paste;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use crate::WordId;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Hash)]
pub struct WordMetadata {
    pub noun: Option<NounData>,
    pub pronoun: Option<PronounData>,
    pub verb: Option<VerbData>,
    pub adjective: Option<AdjectiveData>,
    pub adverb: Option<AdverbData>,
    pub conjunction: Option<ConjunctionData>,
    pub swear: Option<bool>,
    /// The dialect this word belongs to.
    /// If no dialect is defined, it can be assumed that the word is
    /// valid in all dialects of English.
    pub dialect: Option<Dialect>,
    /// Whether the word is a [determiner](https://en.wikipedia.org/wiki/English_determiners).
    #[serde(default = "default_false")]
    pub determiner: bool,
    /// Whether the word is a [preposition](https://www.merriam-webster.com/dictionary/preposition).
    #[serde(default = "default_false")]
    pub preposition: bool,
    /// Whether the word is considered especially common.
    #[serde(default = "default_false")]
    pub common: bool,
    #[serde(default = "default_none")]
    pub derived_from: Option<WordId>,
}

/// Needed for `serde`
fn default_false() -> bool {
    false
}

/// Needed for `serde`
fn default_none<T>() -> Option<T> {
    None
}

macro_rules! generate_metadata_queries {
    ($($category:ident has $($sub:ident),*).*) => {
        paste! {
            pub fn is_likely_homograph(&self) -> bool {
                [self.determiner, self.preposition, $(
                    self.[< is_ $category >](),
                )*].iter().map(|b| *b as u8).sum::<u8>() > 1
            }

            $(
                #[doc = concat!("Checks if the word is definitely a ", stringify!($category), ".")]
                pub fn [< is_ $category >](&self) -> bool {
                    self.$category.is_some()
                }

                $(
                    #[doc = concat!("Checks if the word is definitely a ", stringify!($category), " and more specifically is labeled as (a) ", stringify!($sub), ".")]
                    pub fn [< is_ $sub _ $category >](&self) -> bool {
                        matches!(
                            self.$category,
                            Some([< $category:camel Data >]{
                                [< is_ $sub >]: Some(true),
                                ..
                            })
                        )
                    }


                    #[doc = concat!("Checks if the word is definitely a ", stringify!($category), " and more specifically is labeled as __not__ (a) ", stringify!($sub), ".")]
                    pub fn [< is_not_ $sub _ $category >](&self) -> bool {
                        matches!(
                            self.$category,
                            Some([< $category:camel Data >]{
                                [< is_ $sub >]: Some(false),
                                ..
                            })
                        )
                    }
                )*
            )*
        }
    };
}

impl WordMetadata {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        macro_rules! merge {
            ($a:expr, $b:expr) => {
                match ($a, $b) {
                    (Some(a), Some(b)) => Some(a.or(&b)),
                    (Some(a), None) => Some(a),
                    (None, Some(b)) => Some(b),
                    (None, None) => None,
                }
            };
        }

        Self {
            noun: merge!(self.noun, other.noun),
            pronoun: merge!(self.pronoun, other.pronoun),
            verb: merge!(self.verb, other.verb),
            adjective: merge!(self.adjective, other.adjective),
            adverb: merge!(self.adverb, other.adverb),
            conjunction: merge!(self.conjunction, other.conjunction),
            dialect: self.dialect.or(other.dialect),
            swear: self.swear.or(other.swear),
            determiner: self.determiner || other.determiner,
            preposition: self.preposition || other.preposition,
            common: self.common || other.common,
            derived_from: self.derived_from.or(other.derived_from),
        }
    }

    generate_metadata_queries!(
        noun has proper, plural, possessive.
        pronoun has plural, possessive.
        verb has linking, auxiliary.
        conjunction has.
        adjective has.
        adverb has
    );

    pub fn is_verb_lemma(&self) -> bool {
        matches!(
            self.verb,
            Some(VerbData {
                verb_form: Some(VerbForm::LemmaForm),
                ..
            })
        )
    }

    pub fn is_verb_past_form(&self) -> bool {
        matches!(
            self.verb,
            Some(VerbData {
                verb_form: Some(VerbForm::PastForm),
                ..
            })
        )
    }

    pub fn is_verb_progressive_form(&self) -> bool {
        matches!(
            self.verb,
            Some(VerbData {
                verb_form: Some(VerbForm::ProgressiveForm),
                ..
            })
        )
    }

    pub fn is_verb_third_person_singular_present_form(&self) -> bool {
        matches!(
            self.verb,
            Some(VerbData {
                verb_form: Some(VerbForm::ThirdPersonSingularPresentForm),
                ..
            })
        )
    }

    /// Checks if the word is definitely nominalpro.
    pub fn is_nominal(&self) -> bool {
        self.noun.is_some() || self.pronoun.is_some()
    }

    /// Checks if the word is definitely a nominal and more specifically is labeled as (a) plural.
    pub fn is_plural_nominal(&self) -> bool {
        matches!(
            self.noun,
            Some(NounData {
                is_plural: Some(true),
                ..
            })
        ) || matches!(
            self.pronoun,
            Some(PronounData {
                is_plural: Some(true),
                ..
            })
        )
    }

    /// Checks if the word is definitely a nominal and more specifically is labeled as (a) possessive.
    pub fn is_possessive_nominal(&self) -> bool {
        matches!(
            self.noun,
            Some(NounData {
                is_possessive: Some(true),
                ..
            })
        ) || matches!(
            self.pronoun,
            Some(PronounData {
                is_possessive: Some(true),
                ..
            })
        )
    }

    /// Checks if the word is definitely a nominal and more specifically is labeled as __not__ (a) plural.
    pub fn is_not_plural_nominal(&self) -> bool {
        matches!(
            self.noun,
            Some(NounData {
                is_plural: Some(false),
                ..
            })
        ) || matches!(
            self.pronoun,
            Some(PronounData {
                is_plural: Some(false),
                ..
            })
        )
    }

    /// Checks if the word is definitely a nominal and more specifically is labeled as __not__ (a) possessive.
    pub fn is_not_possessive_nominal(&self) -> bool {
        matches!(
            self.noun,
            Some(NounData {
                is_possessive: Some(false),
                ..
            })
        ) && matches!(
            self.pronoun,
            Some(PronounData {
                is_possessive: Some(false),
                ..
            })
        )
    }

    /// Checks whether a word is _definitely_ a swear.
    pub fn is_swear(&self) -> bool {
        matches!(self.swear, Some(true))
    }

    /// Same thing as [`Self::or`], except in-place rather than a clone.
    pub fn append(&mut self, other: &Self) -> &mut Self {
        *self = self.or(other);
        self
    }
}

// These verb forms are morphological variations, distinct from TAM (Tense-Aspect-Mood)
// Each form can be used in various TAM combinations:
// - Lemma form (infinitive, citation form, dictionary form)
//   Used in infinitives (e.g., "to sleep"), imperatives (e.g., "sleep!"), and with modals (e.g., "will sleep")
// - Past form (past participle and simple past)
//   Used as verbs (e.g., "slept") or adjectives (e.g., "closed door")
// - Progressive form (present participle and gerund)
//   Used as verbs (e.g., "sleeping"), nouns (e.g., "sleeping is important"), or adjectives (e.g., "sleeping dog")
// - Third person singular present (-s/-es)
//   Used for third person singular subjects (e.g., "he sleeps", "she reads")
//
// Important notes:
// 1. English expresses time through auxiliary verbs, not verb form alone
// 2. Irregular verbs can have different forms for past participle and simple past
// 3. Future is always expressed through auxiliary verbs (e.g., "will sleep", "going to sleep")
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Is, Hash)]
pub enum VerbForm {
    LemmaForm,
    PastForm,
    ProgressiveForm,
    ThirdPersonSingularPresentForm,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct VerbData {
    pub is_linking: Option<bool>,
    pub is_auxiliary: Option<bool>,
    pub verb_form: Option<VerbForm>,
}

impl VerbData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        Self {
            is_linking: self.is_linking.or(other.is_linking),
            is_auxiliary: self.is_auxiliary.or(other.is_auxiliary),
            verb_form: self.verb_form.or(other.verb_form),
        }
    }
}

// TODO other noun properties may be worth adding:
// TODO count vs mass; abstract
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct NounData {
    pub is_proper: Option<bool>,
    pub is_plural: Option<bool>,
    pub is_possessive: Option<bool>,
}

impl NounData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        Self {
            is_proper: self.is_proper.or(other.is_proper),
            is_plural: self.is_plural.or(other.is_plural),
            is_possessive: self.is_possessive.or(other.is_possessive),
        }
    }
}

// Person is a property of pronouns; the verb 'be', plus all verbs reflect 3rd person singular with -s
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Is, Hash)]
pub enum Person {
    First,
    Second,
    Third,
}

// case is a property of pronouns
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Is, Hash)]
pub enum Case {
    Subject,
    Object,
}

// TODO for now focused on personal pronouns?
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct PronounData {
    pub is_plural: Option<bool>,
    pub is_possessive: Option<bool>,
    pub person: Option<Person>,
    pub case: Option<Case>,
}

impl PronounData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        Self {
            is_plural: self.is_plural.or(other.is_plural),
            is_possessive: self.is_possessive.or(other.is_possessive),
            person: self.person.or(other.person),
            case: self.case.or(other.case),
        }
    }
}

// Degree is a property of adjectives: positive is not inflected
// Comparative is inflected with -er or comes after the word "more"
// Superlative is inflected with -est or comes after the word "most"
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Is, Hash)]
pub enum Degree {
    Positive,
    Comparative,
    Superlative,
}

// Some adjectives are not comparable so don't have -er or -est forms and can't be used with "more" or "most".
// Some adjectives can only be used "attributively" (before a noun); some only predicatively (after "is" etc.).
// In old grammars words like the articles and determiners are classified as adjectives but behave differently.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct AdjectiveData {
    pub degree: Option<Degree>,
}

impl AdjectiveData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        Self {
            degree: self.degree.or(other.degree),
        }
    }
}

// Adverb can be a "junk drawer" category for words which don't fit the other major categories.
// The typical adverbs are "adverbs of manner", those derived from adjectives in -ly
// other adverbs (time, place, etc) should probably not be considered adverbs for Harper's purposes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct AdverbData {}

impl AdverbData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, _other: &Self) -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct ConjunctionData {}

impl ConjunctionData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, _other: &Self) -> Self {
        Self {}
    }
}

/// A regional dialect.
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, EnumString, Display,
)]
pub enum Dialect {
    American,
    Canadian,
    Australian,
    British,
}
