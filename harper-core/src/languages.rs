use crate::EnglishDialect;
use crate::dialects::portuguese::PortugueseDialect;
use serde::{Deserialize, Serialize};
use std::default::Default;
use strum_macros::Display;
use strum_macros::{EnumCount, EnumDiscriminants, EnumIter, EnumString};

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    PartialOrd,
    Eq,
    Hash,
    EnumCount,
    EnumString,
    EnumIter,
    Display,
    EnumDiscriminants,
)]
pub enum Language {
    English(EnglishDialect),
    Portuguese(PortugueseDialect),
}

impl Default for Language {
    fn default() -> Self {
        Self::English(EnglishDialect::default())
    }
}

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    PartialOrd,
    Eq,
    Hash,
    EnumCount,
    EnumString,
    EnumIter,
    Display,
    EnumDiscriminants,
)]
pub enum LanguageFamily {
    #[default]
    English,
    Portuguese,
}

impl From<Language> for LanguageFamily {
    fn from(value: Language) -> Self {
        match value {
            Language::English(_) => Self::English,
            Language::Portuguese(_) => Self::Portuguese,
        }
    }
}
