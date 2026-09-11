use is_macro::Is;
use serde::{Deserialize, Serialize};

/// Degree is a property of adjectives: positive is not inflected
/// Comparative is inflected with -er or comes after the word "more"
/// Superlative is inflected with -est or comes after the word "most"
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Is, Hash)]
pub enum Degree {
    Positive,
    Comparative,
    Superlative,
}

/// Some adjectives are not comparable so don't have -er or -est forms and can't be used with "more" or "most".
/// Some adjectives can only be used "attributively" (before a noun); some only predicatively (after "is" etc.).
/// In old grammars words like the articles and determiners are classified as adjectives but behave differently.
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
