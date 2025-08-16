use std::fmt::Display;

use is_macro::Is;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumIter};

/// Represents the universal parts of speech as outlined by [universaldependencies.org](https://universaldependencies.org/u/pos/index.html).
#[derive(
    Debug,
    Default,
    Hash,
    Eq,
    PartialEq,
    Clone,
    Copy,
    EnumIter,
    AsRefStr,
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
    Is,
)]
pub enum UPOS {
    /// Adjective
    ADJ,
    /// Adposition
    ADP,
    /// Adverb
    ADV,
    /// Auxiliary
    AUX,
    /// Coordinating conjunction
    CCONJ,
    /// Determiner
    DET,
    /// Interjection
    INTJ,
    /// Noun
    #[default]
    NOUN,
    /// Numeral
    NUM,
    /// Particle
    PART,
    /// Pronoun
    PRON,
    /// Proper noun
    PROPN,
    /// Punctuation
    PUNCT,
    /// Subordinating conjunction
    SCONJ,
    /// Symbol
    SYM,
    /// Verb
    VERB,
}

impl UPOS {
    pub fn from_conllu(other: rs_conllu::UPOS) -> Option<Self> {
        Some(match other {
            rs_conllu::UPOS::ADJ => UPOS::ADJ,
            rs_conllu::UPOS::ADP => UPOS::ADP,
            rs_conllu::UPOS::ADV => UPOS::ADV,
            rs_conllu::UPOS::AUX => UPOS::AUX,
            rs_conllu::UPOS::CCONJ => UPOS::CCONJ,
            rs_conllu::UPOS::DET => UPOS::DET,
            rs_conllu::UPOS::INTJ => UPOS::INTJ,
            rs_conllu::UPOS::NOUN => UPOS::NOUN,
            rs_conllu::UPOS::NUM => UPOS::NUM,
            rs_conllu::UPOS::PART => UPOS::PART,
            rs_conllu::UPOS::PRON => UPOS::PRON,
            rs_conllu::UPOS::PROPN => UPOS::PROPN,
            rs_conllu::UPOS::PUNCT => UPOS::PUNCT,
            rs_conllu::UPOS::SCONJ => UPOS::SCONJ,
            rs_conllu::UPOS::SYM => UPOS::SYM,
            rs_conllu::UPOS::VERB => UPOS::VERB,
            rs_conllu::UPOS::X => return None,
        })
    }

    pub fn is_nominal(&self) -> bool {
        matches!(self, Self::NOUN | Self::PROPN | Self::PRON)
    }
}

impl Display for UPOS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let desc = match self {
            UPOS::ADJ => "Adjective",
            UPOS::ADP => "Adposition",
            UPOS::ADV => "Adverb",
            UPOS::AUX => "Auxiliary",
            UPOS::CCONJ => "Coordinating conjunction",
            UPOS::DET => "Determiner",
            UPOS::INTJ => "Interjection",
            UPOS::NOUN => "Noun",
            UPOS::NUM => "Numeral",
            UPOS::PART => "Particle",
            UPOS::PRON => "Pronoun",
            UPOS::PROPN => "Proper noun",
            UPOS::PUNCT => "Punctuation",
            UPOS::SCONJ => "Subordinating conjunction",
            UPOS::SYM => "Symbol",
            UPOS::VERB => "Verb",
        };
        write!(f, "{desc}")
    }
}
