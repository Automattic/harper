//! English dialects.
//!
//! This module provides type aliases to the original English dialect types
//! from dict_word_metadata.rs to maintain compatibility with the master branch
//! while allowing English to also be a LanguageModule.

use crate::Document;
use crate::dict_word_metadata::{Dialect, DialectFlags};
use crate::language::dialects::dialect_trait::{
    Dialect as DialectTrait, DialectFlags as DialectFlagsTrait,
};

/// Type alias for the original English Dialect enum.
pub type EnglishDialect = Dialect;

/// Type alias for the original English DialectFlags bitflags.
pub type EnglishDialectFlags = DialectFlags;

// Implement the Dialect trait from dialect_trait.rs for the legacy Dialect type
// This allows English to work with the LanguageModule system
impl DialectTrait for Dialect {
    type Flags = DialectFlags;

    fn try_guess_from_document(document: &crate::Document) -> Option<Self> {
        Dialect::try_guess_from_document(document)
    }

    fn try_from_abbr(abbr: &str) -> Option<Self> {
        Dialect::try_from_abbr(abbr)
    }
}

// Implement the DialectFlags trait for the legacy DialectFlags type
impl DialectFlagsTrait<Dialect> for DialectFlags {
    fn is_dialect_enabled(&self, dialect: Dialect) -> bool {
        DialectFlags::is_dialect_enabled(*self, dialect)
    }

    fn is_dialect_enabled_strict(&self, dialect: Dialect) -> bool {
        DialectFlags::is_dialect_enabled_strict(*self, dialect)
    }

    fn from_dialect(dialect: Dialect) -> Self {
        DialectFlags::from_dialect(dialect)
    }

    fn get_most_used_dialects_from_document(document: &Document) -> Self {
        DialectFlags::get_most_used_dialects_from_document(document)
    }
}
