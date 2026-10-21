use is_macro::Is;
use serde::{Deserialize, Serialize};

use crate::Currency;

#[derive(
    Debug, Is, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Default, Hash,
)]
#[serde(tag = "kind")]
pub enum Punctuation {
    /// `°`
    Degree,
    /// `…`
    Ellipsis,
    /// `–`
    EnDash,
    /// `—`
    EmDash,
    /// `&`
    Ampersand,
    /// `.`
    #[default]
    Period,
    /// `!`
    Bang,
    /// `?`
    Question,
    /// `:`
    Colon,
    /// ``;``
    Semicolon,
    /// `"`
    Quote(Quote),
    /// `,`
    Comma,
    /// `-`
    Hyphen,
    /// `[`
    OpenSquare,
    /// `]`
    CloseSquare,
    /// `(`
    OpenRound,
    /// `)`
    CloseRound,
    /// `{`
    OpenCurly,
    /// `}`
    CloseCurly,
    /// `"`
    Hash,
    /// `'`
    Apostrophe,
    /// `%`
    Percent,
    /// `/`
    ForwardSlash,
    /// `\`
    Backslash,
    /// `<`
    LessThan,
    /// `>`
    GreaterThan,
    /// `=`
    Equal,
    /// `*`
    Star,
    /// `~`
    Tilde,
    /// `@`
    At,
    /// `^`
    Caret,
    /// `+`
    Plus,
    Currency(Currency),
    /// `|`
    Pipe,
    /// `_`
    Underscore,
    /// `´`
    Acute,
    /// `‘`
    OpenSingle,
    /// `′`
    SinglePrime,
    /// `″`
    DoublePrime,
    /// `\``,
    Backtick,
}

impl Punctuation {
    pub fn from_char(c: char) -> Option<Punctuation> {
        let punct = match c {
            '´' => Punctuation::Acute,
            '&' => Punctuation::Ampersand,
            '’' => Punctuation::Apostrophe,
            '\'' => Punctuation::Apostrophe,
            '@' => Punctuation::At,
            '\\' => Punctuation::Backslash,
            '!' => Punctuation::Bang,
            '^' => Punctuation::Caret,
            ':' => Punctuation::Colon,
            ',' => Punctuation::Comma,
            '、' => Punctuation::Comma,
            '，' => Punctuation::Comma,
            '°' => Punctuation::Degree,
            '″' => Punctuation::DoublePrime,
            '–' => Punctuation::EnDash,
            '—' => Punctuation::EmDash,
            '…' => Punctuation::Ellipsis,
            '=' => Punctuation::Equal,
            '/' => Punctuation::ForwardSlash,
            '>' => Punctuation::GreaterThan,
            '#' => Punctuation::Hash,
            '-' => Punctuation::Hyphen,
            '<' => Punctuation::LessThan,
            '‘' => Punctuation::OpenSingle,
            '%' => Punctuation::Percent,
            '|' => Punctuation::Pipe,
            '+' => Punctuation::Plus,
            '?' => Punctuation::Question,
            '.' => Punctuation::Period,
            ';' => Punctuation::Semicolon,
            '′' => Punctuation::SinglePrime,
            '*' => Punctuation::Star,
            '~' => Punctuation::Tilde,
            '_' => Punctuation::Underscore,

            '[' => Punctuation::OpenSquare,
            ']' => Punctuation::CloseSquare,
            '{' => Punctuation::OpenCurly,
            '}' => Punctuation::CloseCurly,
            '(' => Punctuation::OpenRound,
            ')' => Punctuation::CloseRound,
            '`' => Punctuation::Backtick,
            _ => Punctuation::Currency(Currency::from_char(c)?),
        };

        Some(punct)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Hash)]
pub struct Quote {
    /// The location of the matching quote, if it exists.
    pub twin_loc: Option<usize>,
}
