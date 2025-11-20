use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumCount, EnumDiscriminants, EnumIter, EnumString, VariantArray};

use crate::{
    Dialect, DialectFlags, EnglishDialect, EnglishDialectFlags,
    dialects::portuguese::{PortugueseDialect, PortugueseDialectFlags},
    languages::{Language, LanguageFamily},
};

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
#[strum_discriminants(derive(VariantArray))] // Apply VariantArray to the discriminants
#[strum_discriminants(name(DialectsEnumKind))]
pub enum DialectsEnum {
    English(EnglishDialect),
    Portuguese(PortugueseDialect),
}

impl Dialect for DialectsEnum {
    type Flags = DialectFlagsEnum;

    fn try_guess_from_document(document: &crate::Document) -> Option<Self> {
        if let Some(english) = EnglishDialect::try_guess_from_document(document) {
            return Some(DialectsEnum::English(english));
        }
        if let Some(portuguese) = PortugueseDialect::try_guess_from_document(document) {
            return Some(DialectsEnum::Portuguese(portuguese));
        }
        None
    }

    fn try_from_abbr(abbr: &str) -> Option<Self> {
        if let Some(english) = EnglishDialect::try_from_abbr(abbr) {
            return Some(DialectsEnum::English(english));
        }
        if let Some(portuguese) = PortugueseDialect::try_from_abbr(abbr) {
            return Some(DialectsEnum::Portuguese(portuguese));
        }

        None
    }
}

impl Default for DialectsEnum {
    fn default() -> Self {
        Self::English(EnglishDialect::default())
    }
}

pub enum DialectFlagsEnum {
    English(EnglishDialectFlags),
    Portuguese(PortugueseDialectFlags),
}
impl DialectFlags<DialectsEnum> for DialectFlagsEnum {
    fn is_dialect_enabled(&self, dialect: DialectsEnum) -> bool {
        match (self, dialect) {
            (
                DialectFlagsEnum::English(english_dialect_flags),
                DialectsEnum::English(english_dialect),
            ) => english_dialect_flags.is_dialect_enabled(english_dialect),
            (
                DialectFlagsEnum::Portuguese(portuguese_dialect_flags),
                DialectsEnum::Portuguese(portuguese_dialect),
            ) => portuguese_dialect_flags.is_dialect_enabled(portuguese_dialect),
            _ => panic!("Trying to get dialect from wrong dialect flags"),
        }
    }

    fn is_dialect_enabled_strict(&self, dialect: DialectsEnum) -> bool {
        match (self, dialect) {
            (
                DialectFlagsEnum::English(english_dialect_flags),
                DialectsEnum::English(english_dialect),
            ) => english_dialect_flags.is_dialect_enabled_strict(english_dialect),
            (
                DialectFlagsEnum::Portuguese(portuguese_dialect_flags),
                DialectsEnum::Portuguese(portuguese_dialect),
            ) => portuguese_dialect_flags.is_dialect_enabled_strict(portuguese_dialect),
            _ => panic!("Trying to get dialect from wrong dialect flags"),
        }
    }

    fn from_dialect(dialect: DialectsEnum) -> Self {
        match dialect {
            DialectsEnum::English(english_dialect) => {
                DialectFlagsEnum::English(EnglishDialectFlags::from_dialect(english_dialect))
            }
            DialectsEnum::Portuguese(portuguese_dialect) => DialectFlagsEnum::Portuguese(
                PortugueseDialectFlags::from_dialect(portuguese_dialect),
            ),
        }
    }

    fn get_most_used_dialects_from_document(
        document: &crate::Document,
        language: Option<LanguageFamily>,
    ) -> Self {
        match language {
            Some(LanguageFamily::English) => DialectFlagsEnum::English(
                EnglishDialectFlags::get_most_used_dialects_from_document(document, language),
            ),
            Some(LanguageFamily::Portuguese) => DialectFlagsEnum::Portuguese(
                PortugueseDialectFlags::get_most_used_dialects_from_document(document, language),
            ),
            None => todo!("Not implemented"),
        }
    }
}
impl Default for DialectFlagsEnum {
    fn default() -> Self {
        Self::English(EnglishDialectFlags::default())
    }
}

impl TryFrom<DialectFlagsEnum> for DialectsEnum {
    type Error = ();

    fn try_from(value: DialectFlagsEnum) -> Result<Self, Self::Error> {
        match value {
            DialectFlagsEnum::English(english_dialect_flags) => {
                Ok(DialectsEnum::English(english_dialect_flags.try_into()?))
            }

            DialectFlagsEnum::Portuguese(portuguese_dialect_flags) => Ok(DialectsEnum::Portuguese(
                portuguese_dialect_flags.try_into()?,
            )),
        }
    }
}
