use is_macro::Is;
use serde::{Deserialize, Serialize};

// nouns can be both singular and plural: "aircraft", "biceps", "fish", "sheep"
// TODO other noun properties may be worth adding: abstract
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct NounData {
    pub is_proper: Option<bool>,
    pub is_singular: Option<bool>,
    pub is_plural: Option<bool>,
    pub is_countable: Option<bool>,
    pub is_mass: Option<bool>,
    pub is_possessive: Option<bool>,
}

impl NounData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        Self {
            is_proper: self.is_proper.or(other.is_proper),
            is_singular: self.is_singular.or(other.is_singular),
            is_plural: self.is_plural.or(other.is_plural),
            is_countable: self.is_countable.or(other.is_countable),
            is_mass: self.is_mass.or(other.is_mass),
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

// TODO for now focused on personal pronouns?
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct PronounData {
    pub is_personal: Option<bool>,
    pub is_singular: Option<bool>,
    pub is_plural: Option<bool>,
    pub is_possessive: Option<bool>,
    pub is_reflexive: Option<bool>,
    pub person: Option<Person>,
    pub is_subject: Option<bool>,
    pub is_object: Option<bool>,
}

impl PronounData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        Self {
            is_personal: self.is_personal.or(other.is_personal),
            is_singular: self.is_singular.or(other.is_singular),
            is_plural: self.is_plural.or(other.is_plural),
            is_possessive: self.is_possessive.or(other.is_possessive),
            is_reflexive: self.is_reflexive.or(other.is_reflexive),
            person: self.person.or(other.person),
            is_subject: self.is_subject.or(other.is_subject),
            is_object: self.is_object.or(other.is_object),
        }
    }
}
