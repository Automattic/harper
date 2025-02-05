mod email_address;
mod hostname;
mod url;

use hostname::lex_hostname_token;
use ordered_float::OrderedFloat;
use url::lex_url;

use self::email_address::lex_email_address;
use crate::char_ext::CharExt;
use crate::punctuation::{Punctuation, Quote};
use crate::{Number, TokenKind, WordMetadata};

#[derive(Debug)]
pub struct FoundToken {
    /// The index of the character __after__ the lexed token
    pub next_index: usize,
    /// Token lexed
    pub token: TokenKind,
}

pub fn lex_token(source: &[char]) -> Option<FoundToken> {
    let lexers = [
        lex_punctuation,
        lex_tabs,
        lex_spaces,
        lex_newlines,
        lex_hex_number, // before lex_number, which would match the initial 0
        lex_number,
        lex_url,
        lex_email_address,
        lex_hostname_token,
        lex_word,
        lex_catch,
    ];

    for lexer in lexers {
        if let Some(f) = lexer(source) {
            return Some(f);
        }
    }

    None
}

fn lex_word(source: &[char]) -> Option<FoundToken> {
    let end = source
        .iter()
        .position(|c| !c.is_english_lingual() && !c.is_numeric())
        .unwrap_or(source.len());

    if end == 0 {
        None
    } else {
        Some(FoundToken {
            next_index: end,
            token: TokenKind::Word(WordMetadata::default()),
        })
    }
}

pub fn lex_number(source: &[char]) -> Option<FoundToken> {
    if source.is_empty() {
        return None;
    }

    if !source[0].is_numeric() {
        return None;
    }

    let end = source
        .iter()
        .enumerate()
        .rev()
        .find_map(|(i, v)| v.is_numeric().then_some(i))?;

    let mut s: String = source[0..end + 1].iter().collect();

    // Find the longest possible valid number
    while !s.is_empty() {
        if let Ok(n) = s.parse::<f64>() {
            let precision = s.chars().rev().position(|c| c == '.').unwrap_or_default();

            return Some(FoundToken {
                token: TokenKind::Number(Number {
                    value: n.into(),
                    suffix: None,
                    radix: 10,
                    precision,
                }),
                next_index: s.len(),
            });
        }

        s.pop();
    }

    None
}

pub fn lex_hex_number(source: &[char]) -> Option<FoundToken> {
    // < 3 to avoid accepting 0x alone
    if source.len() < 3 || source[0] != '0' || source[1] != 'x' || !source[2].is_ascii_hexdigit() {
        return None;
    }

    let mut i = 2;
    let len = source.len();

    while i < len {
        let next = source[i];

        if !next.is_ascii_hexdigit() {
            if !next.is_alphanumeric() {
                break;
            } else {
                return None;
            }
        }

        i += 1;
    }

    let s: String = source[2..i].iter().collect();

    // Should always succeed unless the logic above is broken
    if let Ok(n) = u64::from_str_radix(&s, 16) {
        return Some(FoundToken {
            token: TokenKind::Number(Number {
                value: OrderedFloat(n as f64),
                suffix: None,
                radix: 16,
                precision: 0,
            }),
            next_index: s.len() + 2,
        });
    }

    None
}

fn lex_newlines(source: &[char]) -> Option<FoundToken> {
    let count = source.iter().take_while(|c| **c == '\n').count();

    if count > 0 {
        Some(FoundToken {
            token: TokenKind::Newline(count),
            next_index: count,
        })
    } else {
        None
    }
}

fn lex_tabs(source: &[char]) -> Option<FoundToken> {
    let count = source.iter().take_while(|c| **c == '\t').count();

    if count > 0 {
        Some(FoundToken {
            token: TokenKind::Space(count * 2),
            next_index: count,
        })
    } else {
        None
    }
}

fn lex_spaces(source: &[char]) -> Option<FoundToken> {
    let count = source.iter().take_while(|c| **c == ' ').count();

    if count > 0 {
        Some(FoundToken {
            token: TokenKind::Space(count),
            next_index: count,
        })
    } else {
        None
    }
}

fn lex_punctuation(source: &[char]) -> Option<FoundToken> {
    if let Some(found) = lex_quote(source) {
        return Some(found);
    }

    let c = source.first()?;
    let punct = Punctuation::from_char(*c)?;

    Some(FoundToken {
        next_index: 1,
        token: TokenKind::Punctuation(punct),
    })
}

fn lex_quote(source: &[char]) -> Option<FoundToken> {
    let c = *source.first()?;

    if c == '\"' || c == '“' || c == '”' {
        Some(FoundToken {
            next_index: 1,
            token: TokenKind::Punctuation(Punctuation::Quote(Quote { twin_loc: None })),
        })
    } else {
        None
    }
}

/// Covers cases not covered by the other lints.
fn lex_catch(_source: &[char]) -> Option<FoundToken> {
    Some(FoundToken {
        next_index: 1,
        token: TokenKind::Unlintable,
    })
}

#[cfg(test)]
mod tests {
    use super::lex_hex_number;
    use super::lex_token;
    use super::lex_word;
    use super::{FoundToken, TokenKind};

    #[test]
    fn lexes_cjk_as_unlintable() {
        let source: Vec<_> = "世".chars().collect();
        assert!(lex_word(&source).is_none());
    }

    #[test]
    fn lexes_youtube_as_hostname() {
        let source: Vec<_> = "YouTube.com".chars().collect();
        assert!(matches!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Hostname,
                ..
            })
        ));
    }

    #[test]
    fn lexes_good_hex_numeric() {
        let source: Vec<_> = "0x0".chars().collect();
        assert!(matches!(
            lex_hex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_good_hex_lowercase() {
        let source: Vec<_> = "0xa".chars().collect();
        assert!(matches!(
            lex_hex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_good_hex_uppercase() {
        let source: Vec<_> = "0xF".chars().collect();
        assert!(matches!(
            lex_hex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_good_hex_mixed_case() {
        let source: Vec<_> = "0xaF".chars().collect();
        assert!(matches!(
            lex_hex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_good_hex_lowercase_long() {
        let source: Vec<_> = "0x0123456789abcdef".chars().collect();
        assert!(matches!(
            lex_hex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_good_hex_uppercase_long() {
        let source: Vec<_> = "0x0123456789ABCDEF".chars().collect();
        assert!(matches!(
            lex_hex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn does_not_lex_prefix_only() {
        let source: Vec<_> = "0x".chars().collect();
        assert!(lex_hex_number(&source).is_none());
    }

    #[test]
    fn does_not_lex_bad_alphabetic() {
        let source: Vec<_> = "0xg".chars().collect();
        assert!(lex_hex_number(&source).is_none());
    }

    #[test]
    fn does_not_lex_bad_after_good() {
        let source: Vec<_> = "0x123g".chars().collect();
        assert!(lex_hex_number(&source).is_none());
    }

    #[test]
    fn does_not_lex_uppercase_prefix() {
        let source: Vec<_> = "0Xf00d".chars().collect();
        assert!(lex_hex_number(&source).is_none());
    }
}
