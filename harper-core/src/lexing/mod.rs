mod email_address;
mod hostname;
mod url;

use hostname::lex_hostname_token;
use ordered_float::OrderedFloat;
use url::lex_url;

use self::email_address::lex_email_address;
use crate::char_ext::CharExt;
use crate::punctuation::{Punctuation, Quote};
use crate::{Number, TokenKind};

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
        lex_hex_number,  // before lex_number, which would match the initial 0
        lex_long_decade, // before lex_number, which would match the digits up to the -s
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
            token: TokenKind::Word(None),
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

pub fn lex_long_decade(source: &[char]) -> Option<FoundToken> {
    // lex 4-digit decades in their plural such as: 1980s 1990s 2000s 2020s
    if source.len() < 5 {
        return None;
    }
    if source[0] != '1' && source[0] != '2' {
        return None;
    }
    if !source[1].is_ascii_digit() {
        return None;
    }
    if !source[2].is_ascii_digit() {
        return None;
    }
    if source[3] != '0' {
        return None;
    }
    if source[4] != 's' {
        return None;
    }

    Some(FoundToken {
        token: TokenKind::Decade,
        next_index: 5,
    })
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
    use super::lex_long_decade;
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

    #[test]
    fn lexes_20c_decade() {
        let source: Vec<_> = "1980s".chars().collect();
        assert!(matches!(
            lex_long_decade(&source),
            Some(FoundToken {
                token: TokenKind::Decade,
                ..
            })
        ));
    }

    #[test]
    fn lexes_21c_decade() {
        let source: Vec<_> = "2020s".chars().collect();
        assert!(matches!(
            lex_long_decade(&source),
            Some(FoundToken {
                token: TokenKind::Decade,
                ..
            })
        ));
    }

    #[test]
    fn lexes_ancient_decade() {
        let source: Vec<_> = "1010s".chars().collect();
        assert!(matches!(
            lex_long_decade(&source),
            Some(FoundToken {
                token: TokenKind::Decade,
                ..
            })
        ));
    }

    #[test]
    fn lexes_word_before_decade() {
        let source: Vec<_> = "late 1980s".chars().collect();
        assert!(matches!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Word(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_word_after_decade() {
        let source: Vec<_> = "1980s and".chars().collect();
        assert!(matches!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Decade,
                ..
            })
        ));
    }

    #[test]
    fn doesnt_lex_far_future_decade() {
        let source: Vec<_> = "3190s".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_too_ancient_decade() {
        let source: Vec<_> = "100s".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_0_prefixed_decade() {
        let source: Vec<_> = "0100s".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_uppercase_decade() {
        let source: Vec<_> = "2000S".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_overlong_decade() {
        let source: Vec<_> = "20000s".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_apostrophe_long_decade() {
        let source: Vec<_> = "2020's".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_bad_apostrophe_short_decade() {
        let source: Vec<_> = "80's".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_good_apostrophe_short_decade() {
        let source: Vec<_> = "'90s".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn accepts_sentence_with_decade() {
        let sentence: Vec<_> = "To the early 1990s there were a lot of Movies where the bad guys were former Russian intelligence agents.".chars().collect();
        let expected_tokens = [
            TokenKind::Word(None),
            TokenKind::Space(1),
            TokenKind::Word(None),
            TokenKind::Space(1),
            TokenKind::Word(None),
            TokenKind::Space(1),
            TokenKind::Decade,
        ];

        let mut next_index = 0;

        for expected_token in expected_tokens.iter() {
            if next_index >= sentence.len() {
                break; // Exit if we've processed the entire source
            }

            let token = lex_token(&sentence[next_index..]).expect("Failed to lex token");
            assert_eq!(token.token, *expected_token);
            next_index += token.next_index;
        }
    }

    #[test]
    fn rejects_sentence_with_number() {
        let sentence: Vec<_> = "To the early 1990s there were a lot of Movies where the bad guys were former Russian intelligence agents.".chars().collect();
        let expected_tokens = [
            TokenKind::Word(None),
            TokenKind::Space(1),
            TokenKind::Word(None),
            TokenKind::Space(1),
            TokenKind::Word(None),
            TokenKind::Space(1),
            TokenKind::Number(Default::default()),
        ];

        let mut next_index = 0;

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            if next_index >= sentence.len() {
                break; // Exit if we've processed the entire source
            }

            let token = lex_token(&sentence[next_index..]).expect("Failed to lex token");

            if i < 6 {
                assert_eq!(token.token, *expected_token);
            } else {
                assert_ne!(token.token, *expected_token);
            }

            next_index += token.next_index;
        }
    }
}
