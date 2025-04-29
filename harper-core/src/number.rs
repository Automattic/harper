use std::fmt::Display;

use is_macro::Is;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd)]
pub struct Number {
    pub value: OrderedFloat<f64>,
    pub suffix: Option<OrdinalSuffix>,
    pub radix: u32,
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
                write!(f, "{}", c)?;
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

        match integer % 10 {
            0 => Some(Self::Th),
            1 => Some(Self::St),
            2 => Some(Self::Nd),
            3 => Some(Self::Rd),
            4 => Some(Self::Th),
            5 => Some(Self::Th),
            6 => Some(Self::Th),
            7 => Some(Self::Th),
            8 => Some(Self::Th),
            9 => Some(Self::Th),
            _ => None,
        }
    }

    pub fn to_chars(self) -> Vec<char> {
        match self {
            OrdinalSuffix::Th => vec!['t', 'h'],
            OrdinalSuffix::St => vec!['s', 't'],
            OrdinalSuffix::Nd => vec!['n', 'd'],
            OrdinalSuffix::Rd => vec!['r', 'd'],
        }
    }

    /// Check the first several characters in a buffer to see if it matches a
    /// number suffix.
    pub fn from_chars(chars: &[char]) -> Option<Self> {
        if chars.len() != 2 {
            return None;
        }

        match (chars[0], chars[1]) {
            ('t', 'h') => Some(OrdinalSuffix::Th),
            ('T', 'h') => Some(OrdinalSuffix::Th),
            ('t', 'H') => Some(OrdinalSuffix::Th),
            ('T', 'H') => Some(OrdinalSuffix::Th),
            ('s', 't') => Some(OrdinalSuffix::St),
            ('S', 't') => Some(OrdinalSuffix::St),
            ('s', 'T') => Some(OrdinalSuffix::St),
            ('S', 'T') => Some(OrdinalSuffix::St),
            ('n', 'd') => Some(OrdinalSuffix::Nd),
            ('N', 'd') => Some(OrdinalSuffix::Nd),
            ('n', 'D') => Some(OrdinalSuffix::Nd),
            ('N', 'D') => Some(OrdinalSuffix::Nd),
            ('r', 'd') => Some(OrdinalSuffix::Rd),
            ('R', 'd') => Some(OrdinalSuffix::Rd),
            ('r', 'D') => Some(OrdinalSuffix::Rd),
            ('R', 'D') => Some(OrdinalSuffix::Rd),
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
