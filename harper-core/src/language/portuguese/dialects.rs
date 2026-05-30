//! Portuguese dialect support.

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumCount, EnumIter, EnumString, VariantArray};

/// Portuguese dialects supported by Harper.
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
pub enum PortugueseDialect {
    /// European Portuguese (Portugal)
    European,
    /// Brazilian Portuguese (Brazil) - default
    #[default]
    Brazilian,
    /// African Portuguese (Angola, Mozambique, etc.)
    African,
}

impl PortugueseDialect {
    /// Tries to get a dialect from its abbreviation.
    #[must_use]
    pub fn try_from_abbr(abbr: &str) -> Option<Self> {
        match abbr {
            "PT" | "European" => Some(Self::European),
            "BR" | "Brazilian" => Some(Self::Brazilian),
            "AF" | "African" => Some(Self::African),
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
            PortugueseDialect::try_from_abbr("PT"),
            Some(PortugueseDialect::European)
        );
        assert_eq!(
            PortugueseDialect::try_from_abbr("BR"),
            Some(PortugueseDialect::Brazilian)
        );
        assert_eq!(
            PortugueseDialect::try_from_abbr("AF"),
            Some(PortugueseDialect::African)
        );
        assert_eq!(
            PortugueseDialect::try_from_abbr("European"),
            Some(PortugueseDialect::European)
        );
        assert_eq!(PortugueseDialect::try_from_abbr("XY"), None);
    }

    #[test]
    fn test_default_dialect() {
        assert_eq!(PortugueseDialect::default(), PortugueseDialect::Brazilian);
    }
}
