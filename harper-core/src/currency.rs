use is_macro::Is;
use serde::{Deserialize, Serialize};

use crate::Number;

/// A national or international currency
#[derive(Debug, Is, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Hash)]
pub enum Currency {
    // $
    Dollar,
    // ¢
    Cent,
    // €
    Euro,
    // ₽
    Ruble,
    // ₺
    Lira,
    // £
    Pound,
    // ¥
    Yen,
    // ฿
    Baht,
    // ₩
    Won,
    // ₭,
    Kip,
}

impl Currency {
    pub fn from_char(c: char) -> Option<Self> {
        let cur = match c {
            '$' => Self::Dollar,
            '¢' => Self::Cent,
            '€' => Self::Euro,
            '₽' => Self::Ruble,
            '₺' => Self::Lira,
            '£' => Self::Pound,
            '¥' => Self::Yen,
            '฿' => Self::Baht,
            '₩' => Self::Won,
            '₭' => Self::Kip,
            _ => return None,
        };

        Some(cur)
    }

    pub fn to_char(&self) -> char {
        match self {
            Self::Dollar => '$',
            Self::Cent => '¢',
            Self::Euro => '€',
            Self::Ruble => '₽',
            Self::Lira => '₺',
            Self::Pound => '£',
            Self::Yen => '¥',
            Self::Baht => '฿',
            Self::Won => '₩',
            Self::Kip => '₭',
        }
    }

    /// Format an amount of the specific currency.
    pub fn format_amount(&self, amount: &Number) -> String {
        let c = self.to_char();

        let amount = amount.to_string();

        match self {
            Currency::Dollar => format!("{c}{amount}"),
            Currency::Cent => format!("{amount}{c}"),
            Currency::Euro => format!("{c}{amount}"),
            Currency::Ruble => format!("{amount} {c}"),
            Currency::Lira => format!("{amount} {c}"),
            Currency::Pound => format!("{c}{amount}"),
            Currency::Yen => format!("{c} {amount}"),
            Currency::Baht => format!("{amount} {c}"),
            Currency::Won => format!("{c} {amount}"),
            Currency::Kip => format!("{c}{amount}"),
        }
    }
}
