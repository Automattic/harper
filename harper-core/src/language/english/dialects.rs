//! English dialects.
//!
//! This module provides the original English dialect types from dict_word_metadata.rs
//! to integrate with the language module system.

use crate::Document;
use crate::dict_word_metadata::{Dialect, DialectFlags};
use crate::language::dialects::dialect_trait::{
    Dialect as DialectTrait, DialectFlags as DialectFlagsTrait,
};

/// Type alias for the original English Dialect enum.
pub type EnglishDialect = Dialect;

/// Type alias for the original English DialectFlags bitflags.
pub type EnglishDialectFlags = DialectFlags;

// The language module system needs a default dialect per language, but the core
// `Dialect` enum deliberately has no inherent notion of one. Supplying it here
// rather than deriving `Default` in `dict_word_metadata.rs` keeps the core file
// identical to upstream.
//
// Clippy suggests deriving this instead. Doing so would mean adding
// `#[derive(Default)]` and `#[default]` to the enum in `dict_word_metadata.rs`,
// which is precisely the upstream churn this impl exists to avoid.
#[allow(clippy::derivable_impls)]
impl Default for Dialect {
    fn default() -> Self {
        Dialect::American
    }
}

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
