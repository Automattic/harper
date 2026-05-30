//! German dialect support.

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumCount, EnumIter, EnumString, VariantArray};

/// German dialects supported by Harper.
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
    VariantArray,
)]
pub enum GermanDialect {
    /// Standard German (Deutschland)
    #[default]
    Standard,
    /// Austrian German (Österreich)
    Austrian,
    /// Swiss German (Schweiz)
    Swiss,
}

impl GermanDialect {
    /// Tries to get a dialect from its abbreviation.
    #[must_use]
    pub fn try_from_abbr(abbr: &str) -> Option<Self> {
        match abbr {
            "DE" | "Standard" => Some(Self::Standard),
            "AT" | "Austrian" => Some(Self::Austrian),
            "CH" | "Swiss" => Some(Self::Swiss),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialect_abbreviations() {
        assert_eq!(
            GermanDialect::try_from_abbr("DE"),
            Some(GermanDialect::Standard)
        );
        assert_eq!(
            GermanDialect::try_from_abbr("AT"),
            Some(GermanDialect::Austrian)
        );
        assert_eq!(
            GermanDialect::try_from_abbr("CH"),
            Some(GermanDialect::Swiss)
        );
        assert_eq!(
            GermanDialect::try_from_abbr("Standard"),
            Some(GermanDialect::Standard)
        );
        assert_eq!(GermanDialect::try_from_abbr("XY"), None);
    }
}
