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

#[derive(Debug, Eq, PartialEq)]
pub struct FoundToken {
    /// The index of the character __after__ the lexed token
    pub next_index: usize,
    /// Token lexed
    pub token: TokenKind,
}

pub fn lex_token(source: &[char]) -> Option<FoundToken> {
    let lexers = [
        lex_regexish,
        lex_punctuation,
        lex_tabs,
        lex_spaces,
        lex_newlines,
        lex_plural_digit, // Before lex_number, which would match the initial digit
        lex_hex_number,   // Before lex_number, which would match the initial 0
        lex_long_decade,  // Before lex_number, which would match the digits up to the -s
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
        .position(|c| !c.is_english_lingual() && !c.is_ascii_digit())
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
        .find_map(|(i, v)| v.is_ascii_digit().then_some(i))?;

    let mut s: String = source[0..end + 1].iter().collect();

    // Find the longest possible valid number
    while !s.is_empty() {
        if let Ok(n) = s.parse::<f64>() {
            let precision = s.chars().rev().position(|c| c == '.').unwrap_or_default();

            if !s.ends_with('.') {
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
        }

        s.pop();
    }

    None
}

// Often in comments we mention partial- or pseudo- regexes. Here's an example from Ghidra:
// ([a-z0-9]+ only) - We previously flagged just the z0 in the middle of it.
pub fn lex_regexish(src: &[char]) -> Option<FoundToken> {
    let l = src.len();
    let mut i = 0;

    if i >= l || src[i] != '[' {
        return None;
    }
    i += 1;

    loop {
        if i >= l || !src[i].is_alphanumeric() {
            return None;
        }
        i += 1;
        if i < l && src[i] == '-' {
            i += 1;
            if i >= l || !src[i].is_alphanumeric() {
                return None;
            }
            i += 1;
        }

        if i >= l || src[i] != ']' {
            continue;
        }
        break;
    }

    Some(FoundToken {
        token: TokenKind::Regexish,
        next_index: i + 1,
    })
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

pub fn lex_plural_digit(src: &[char]) -> Option<FoundToken> {
    // Issue #774
    let l = src.len();
    let mut i = 0;

    if src.is_empty() || !src[i].is_ascii_alphanumeric() {
        return None;
    }
    i += 1;

    if l > i && src[i] == '\'' {
        i += 1;
    }

    if l > i && src[i] == 's' {
        i += 1;

        if l == i || !src[i].is_ascii_alphanumeric() {
            return Some(FoundToken {
                token: TokenKind::Word(None),
                next_index: i,
            });
        }
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
    use crate::Punctuation;
    use crate::char_string::char_string;
    use crate::lexing::lex_plural_digit;

    use super::lex_hex_number;
    use super::lex_long_decade;
    use super::lex_number;
    use super::lex_token;
    use super::lex_word;
    use super::{FoundToken, TokenKind};

    // test various kinds of number
    #[test]
    fn lexes_0() {
        let source: Vec<_> = "0".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_0_point_0() {
        let source: Vec<_> = "0.0".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_00() {
        let source: Vec<_> = "00".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[ignore = "Negative numbers are not yet supported"]
    fn lexes_negative_1() {
        let source: Vec<_> = "-1".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[ignore = "Positive numbers with a leading + are not supported"]
    fn lexes_positive_1() {
        let source: Vec<_> = "+1".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_pi() {
        let source: Vec<_> = "3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_speed_of_light() {
        let source: Vec<_> = "3.00e8".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn doesnt_lex_cjk_numeral() {
        let source: Vec<_> = "二".chars().collect();
        assert!(lex_number(&source).is_none());
    }

    #[test]
    fn doesnt_lex_thai_digit() {
        let source: Vec<_> = "๑".chars().collect();
        assert!(lex_number(&source).is_none());
    }

    #[test]
    fn lexes_cjk_as_unlintable() {
        let source: Vec<_> = "世".chars().collect();
        assert!(lex_word(&source).is_none());
    }

    #[test]
    fn lexes_youtube_as_hostname() {
        let source: Vec<_> = "YouTube.com".chars().collect();
        assert_eq!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Hostname,
                next_index: source.len()
            })
        );
    }

    #[test]
    fn doesnt_lex_regex_mini_range() {
        let source: Vec<_> = "[]".chars().collect();
        assert!(!matches!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Regexish,
                next_index: 2
            })
        ))
    }

    #[test]
    fn lexes_regex_one_letter() {
        let source: Vec<_> = "[a]".chars().collect();
        assert_eq!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Regexish,
                next_index: 3
            })
        );
    }

    #[test]
    fn lexes_regex_two_letters() {
        let source: Vec<_> = "[az]".chars().collect();
        assert_eq!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Regexish,
                next_index: 4
            })
        );
    }

    #[test]
    fn lexes_regex_digits() {
        let source: Vec<_> = "[123]".chars().collect();
        assert_eq!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Regexish,
                next_index: 5
            })
        );
    }

    #[test]
    fn lexes_regex_two_alphanumeric() {
        let source: Vec<_> = "[a0b1c2]".chars().collect();
        assert_eq!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Regexish,
                next_index: 8
            })
        );
    }

    #[test]
    fn lexes_regex_one_range() {
        let source: Vec<_> = "[a-z]".chars().collect();
        assert_eq!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Regexish,
                next_index: 5
            })
        );
    }

    #[test]
    fn lexes_regex_letter_plus_range() {
        let source: Vec<_> = "[ax-z]".chars().collect();
        assert_eq!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Regexish,
                next_index: 6
            })
        );
    }

    #[test]
    fn lexes_regex_range_plus_letter() {
        let source: Vec<_> = "[a-cz]".chars().collect();
        assert_eq!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Regexish,
                next_index: 6
            })
        );
    }

    #[test]
    fn lexes_regex_two_ranges() {
        let source: Vec<_> = "[a-cx-z]".chars().collect();
        assert_eq!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Regexish,
                next_index: 8
            })
        );
    }

    #[test]
    fn doesnt_lex_regex_broken_two_ranges() {
        // You can't end a range and start a range with a single letter
        let source: Vec<_> = "[a-x-z]".chars().collect();
        assert_eq!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Punctuation(Punctuation::OpenSquare),
                next_index: 1
            })
        );
    }

    #[test]
    fn doesnt_lex_regex_hyphen_at_start() {
        let source: Vec<_> = "[a-]".chars().collect();
        assert!(!matches!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Regexish,
                ..
            })
        ));
    }

    #[test]
    fn doesnt_lex_regex_hyphen_at_end() {
        let source: Vec<_> = "[-z]".chars().collect();
        assert!(!matches!(
            lex_token(&source),
            Some(FoundToken {
                token: TokenKind::Regexish,
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
    fn lexes_0s() {
        let source: Vec<_> = "0s".chars().collect();
        assert!(matches!(
            // lex_token(&source),
            lex_plural_digit(&source),
            Some(FoundToken {
                token: TokenKind::Word(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_1_apostrophe_s() {
        let source: Vec<_> = "1's".chars().collect();
        assert!(matches!(
            // lex_token(&source),
            lex_plural_digit(&source),
            Some(FoundToken {
                token: TokenKind::Word(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_0s_and_1s() {
        let source: Vec<_> = "0s and 1s".chars().collect();
        assert!(matches!(
            // lex_token(&source),
            lex_plural_digit(&source),
            Some(FoundToken {
                token: TokenKind::Word(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_1s_and_0s_apostrophes() {
        let source: Vec<_> = "1's and 0's".chars().collect();
        assert!(matches!(
            // lex_token(&source),
            lex_plural_digit(&source),
            Some(FoundToken {
                token: TokenKind::Word(_),
                ..
            })
        ));
    }

    #[test]
    fn doesnt_lex_0s_joined_letter() {
        let source: Vec<_> = "0ss".chars().collect();
        assert!(lex_plural_digit(&source).is_none());
    }

    #[test]
    fn doesnt_lex_1s_apostrophe_joined_number() {
        let source: Vec<_> = "1's1".chars().collect();
        assert!(lex_plural_digit(&source).is_none());
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

    #[test]
    fn issue_1010() {
        let source = char_string!("3.");

        let tok = lex_number(&source).unwrap();
        assert_eq!(tok.next_index, 1);
    }

    #[test]
    fn lexes_full_number() {
        let source = char_string!("3.0");

        let tok = lex_number(&source).unwrap();
        assert_eq!(tok.next_index, 3);
    }
}
