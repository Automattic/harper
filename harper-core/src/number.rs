use std::fmt::Display;

use is_macro::Is;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

/// Represents a written number.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd)]
pub struct Number {
    /// The actual value of the number
    pub value: OrderedFloat<f64>,
    /// Whether it contains a suffix (like the 1__st__ element).
    pub suffix: Option<OrdinalSuffix>,
    /// What base it is in (hex v.s. decimal, for example).
    pub radix: u32,
    /// The level of precision the number is formatted with.
    pub precision: usize,
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.radix == 16 {
            write!(f, "0x{:X}", self.value.0 as u64)?;
        } else {
            write!(f, "{:.*}", self.precision, self.value.0)?;
        }

        if let Some(suffix) = self.suffix {
            for c in suffix.to_chars() {
                write!(f, "{c}")?;
            }
        }

        Ok(())
    }
}

#[derive(
    Debug, Serialize, Deserialize, Default, PartialEq, PartialOrd, Clone, Copy, Is, Hash, Eq,
)]
pub enum OrdinalSuffix {
    #[default]
    Th,
    St,
    Nd,
    Rd,
}

impl OrdinalSuffix {
    pub fn correct_suffix_for(number: impl Into<f64>) -> Option<Self> {
        let number = number.into();

        if number < 0.0 || number - number.floor() > f64::EPSILON || number > u64::MAX as f64 {
            return None;
        }

        let integer = number as u64;

        if let 11..=13 = integer % 100 {
            return Some(Self::Th);
        };

        Some(match integer % 10 {
            0 | 4..=9 => Self::Th,
            1 => Self::St,
            2 => Self::Nd,
            3 => Self::Rd,
            _ => unreachable!(),
        })
    }

    pub const fn to_chars(self) -> &'static [char] {
        match self {
            OrdinalSuffix::Th => &['t', 'h'],
            OrdinalSuffix::St => &['s', 't'],
            OrdinalSuffix::Nd => &['n', 'd'],
            OrdinalSuffix::Rd => &['r', 'd'],
        }
    }

    /// Check the characters in a buffer to see if it matches a number suffix.
    pub fn from_chars(chars: &[char]) -> Option<Self> {
        let lower_chars: [char; 2] = chars.iter().map(char::to_ascii_lowercase).collect_array()?;

        match lower_chars {
            ['t', 'h'] => Some(OrdinalSuffix::Th),
            ['s', 't'] => Some(OrdinalSuffix::St),
            ['n', 'd'] => Some(OrdinalSuffix::Nd),
            ['r', 'd'] => Some(OrdinalSuffix::Rd),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use ordered_float::OrderedFloat;

    use crate::OrdinalSuffix;

    use super::Number;

    #[test]
    fn hex_fifteen() {
        assert_eq!(
            Number {
                value: OrderedFloat(15.0),
                suffix: None,
                radix: 16,
                precision: 0
            }
            .to_string(),
            "0xF"
        )
    }

    #[test]
    fn decimal_fifteen() {
        assert_eq!(
            Number {
                value: OrderedFloat(15.0),
                suffix: None,
                radix: 10,
                precision: 0
            }
            .to_string(),
            "15"
        )
    }

    #[test]
    fn decimal_fifteen_suffix() {
        assert_eq!(
            Number {
                value: OrderedFloat(15.0),
                suffix: Some(OrdinalSuffix::Th),
                radix: 10,
                precision: 0
            }
            .to_string(),
            "15th"
        )
    }

    #[test]
    fn decimal_fifteen_and_a_half() {
        assert_eq!(
            Number {
                value: OrderedFloat(15.5),
                suffix: None,
                radix: 10,
                precision: 2
            }
            .to_string(),
            "15.50"
        )
    }

    #[test]
    fn issue_1051() {
        let word = "story".chars().collect_vec();
        assert_eq!(None, OrdinalSuffix::from_chars(&word));
    }
}
