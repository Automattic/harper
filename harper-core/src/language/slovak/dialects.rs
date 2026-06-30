//! Slovak dialect support.

use crate::language::dialects::dialect_trait::{Dialect, DialectFlags};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum::{EnumCount, VariantArray};
use strum_macros::{Display, EnumCount, EnumIter, EnumString, VariantArray};

use crate::Document;

/// Slovak dialects supported by Harper.
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
    Default,
    EnumCount,
    EnumString,
    EnumIter,
    Display,
    VariantArray,
)]
pub enum SlovakDialect {
    /// Standard Slovak (Slovensko)
    #[default]
    Standard = 1 << 0,
}

impl SlovakDialect {
    /// Tries to get a dialect from its abbreviation.
    #[must_use]
    pub fn try_from_abbr(abbr: &str) -> Option<Self> {
        match abbr {
            "SK" | "Standard" | "Slovak" | "Slovensko" => Some(Self::Standard),
            _ => None,
        }
    }
}

impl Dialect for SlovakDialect {
    type Flags = SlovakDialectFlags;

    /// Tries to guess the dialect used in the document by finding which dialect is used the most.
    /// Returns `None` if it fails to find a single dialect that is used the most.
    fn try_guess_from_document(document: &Document) -> Option<Self> {
        Self::try_from(SlovakDialectFlags::get_most_used_dialects_from_document(
            document,
        ))
        .ok()
    }

    fn try_from_abbr(abbr: &str) -> Option<Self> {
        Self::try_from_abbr(abbr)
    }
}

impl TryFrom<SlovakDialectFlags> for SlovakDialect {
    type Error = ();

    /// Attempts to convert `DialectFlags` to a single `Dialect`.
    ///
    /// # Errors
    ///
    /// Will return `Err` if more than one dialect is enabled or if an undefined dialect is
    /// enabled.
    fn try_from(dialect_flags: SlovakDialectFlags) -> Result<Self, Self::Error> {
        // Ensure only one dialect is enabled before converting.
        if dialect_flags.bits().count_ones() == 1 {
            if dialect_flags.is_dialect_enabled_strict(SlovakDialect::Standard) {
                Ok(SlovakDialect::Standard)
            } else {
                Err(())
            }
        } else {
            // More than one dialect enabled; can't soundly convert.
            Err(())
        }
    }
}

// The underlying type used for DialectFlags.
type DialectFlagsUnderlyingType = u8;

bitflags::bitflags! {
    /// A collection of bit flags used to represent enabled Slovak dialects.
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
    #[serde(transparent)]
    pub struct SlovakDialectFlags: DialectFlagsUnderlyingType {
        const STANDARD = SlovakDialect::Standard as DialectFlagsUnderlyingType;
    }
}

impl DialectFlags<SlovakDialect> for SlovakDialectFlags {
    /// Checks if the provided dialect is enabled.
    /// If no dialect is explicitly enabled, it is assumed that all dialects are enabled.
    fn is_dialect_enabled(&self, dialect: SlovakDialect) -> bool {
        self.is_empty() || self.intersects(Self::from_dialect(dialect))
    }

    /// Checks if the provided dialect is ***explicitly*** enabled.
    fn is_dialect_enabled_strict(&self, dialect: SlovakDialect) -> bool {
        self.intersects(Self::from_dialect(dialect))
    }

    /// Constructs a `DialectFlags` from the provided `Dialect`.
    fn from_dialect(dialect: SlovakDialect) -> Self {
        let Some(out) = Self::from_bits(dialect as DialectFlagsUnderlyingType) else {
            panic!("The '{dialect}' dialect isn't defined in DialectFlags!");
        };
        out
    }

    /// Gets the most commonly used dialect(s) in the document.
    fn get_most_used_dialects_from_document(document: &Document) -> Self {
        // For now, Slovak has only one dialect, so we return the standard flag
        // This will be enhanced when more dialects are added
        Self::from_dialect(SlovakDialect::Standard)
    }
}

impl Default for SlovakDialectFlags {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialect_abbreviations() {
        assert_eq!(
            SlovakDialect::try_from_abbr("SK"),
            Some(SlovakDialect::Standard)
        );
        assert_eq!(
            SlovakDialect::try_from_abbr("Standard"),
            Some(SlovakDialect::Standard)
        );
        assert_eq!(
            SlovakDialect::try_from_abbr("Slovak"),
            Some(SlovakDialect::Standard)
        );
        assert_eq!(
            SlovakDialect::try_from_abbr("Slovensko"),
            Some(SlovakDialect::Standard)
        );
        assert_eq!(SlovakDialect::try_from_abbr("XY"), None);
    }
}