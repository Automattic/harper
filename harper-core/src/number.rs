use std::fmt::Display;

use is_macro::Is;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd)]
pub struct Number {
    pub value: OrderedFloat<f64>,
    pub suffix: Option<NumberSuffix>,
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
pub enum NumberSuffix {
    #[default]
    Th,
    St,
    Nd,
    Rd,
}

impl NumberSuffix {
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
            NumberSuffix::Th => vec!['t', 'h'],
            NumberSuffix::St => vec!['s', 't'],
            NumberSuffix::Nd => vec!['n', 'd'],
            NumberSuffix::Rd => vec!['r', 'd'],
        }
    }

    /// Check the first several characters in a buffer to see if it matches a
    /// number suffix.
    pub fn from_chars(chars: &[char]) -> Option<Self> {
        if chars.len() < 2 {
            return None;
        }

        match (chars[0], chars[1]) {
            ('t', 'h') => Some(NumberSuffix::Th),
            ('T', 'h') => Some(NumberSuffix::Th),
            ('t', 'H') => Some(NumberSuffix::Th),
            ('T', 'H') => Some(NumberSuffix::Th),
            ('s', 't') => Some(NumberSuffix::St),
            ('S', 't') => Some(NumberSuffix::St),
            ('s', 'T') => Some(NumberSuffix::St),
            ('S', 'T') => Some(NumberSuffix::St),
            ('n', 'd') => Some(NumberSuffix::Nd),
            ('N', 'd') => Some(NumberSuffix::Nd),
            ('n', 'D') => Some(NumberSuffix::Nd),
            ('N', 'D') => Some(NumberSuffix::Nd),
            ('r', 'd') => Some(NumberSuffix::Rd),
            ('R', 'd') => Some(NumberSuffix::Rd),
            ('r', 'D') => Some(NumberSuffix::Rd),
            ('R', 'D') => Some(NumberSuffix::Rd),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use crate::NumberSuffix;

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
                suffix: Some(NumberSuffix::Th),
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
}
