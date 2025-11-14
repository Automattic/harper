use crate::TokenKind;

use super::FoundToken;

/// Lex a hostname token.
pub fn lex_hostname_token(source: &[char]) -> Option<FoundToken> {
    let len = lex_hostname(source)?;

    // Might be word, just skip it.
    if len <= 1 {
        return None;
    }

    if !source.get(1..len - 1)?.contains(&'.') {
        return None;
    }

    if source.get(len - 1) == Some(&'.') {
        return None;
    }

    // For the sake of semantics and downstream grammar checking.
    if !ends_with_common_tld(&source[0..len]) {
        return None;
    }

    Some(FoundToken {
        next_index: len,
        token: TokenKind::Hostname,
    })
}

pub fn lex_hostname(source: &[char]) -> Option<usize> {
    let mut passed_chars = 0;

    // The beginning has different requirements from the rest of the hostname.
    let first = source.first()?;

    if !matches!(first,  'A'..='Z' |  'a'..='z' | '0'..='9' ) {
        return None;
    }

    for label in source.split(|c| *c == '.') {
        for c in label {
            passed_chars += 1;
            if !matches!(c,  'A'..='Z' |  'a'..='z' | '0'..='9' | '-') {
                return Some(passed_chars - 1);
            }
        }

        passed_chars += 1;
    }

    if passed_chars == 0 {
        None
    } else {
        Some(passed_chars - 1)
    }
}

const COMMON_TLDS: &[&[char]] = &[
    &['c', 'o', 'm'],
    &['n', 'e', 't'],
    &['o', 'r', 'g'],
    &['e', 'd', 'u'],
    &['g', 'o', 'v'],
    &['m', 'i', 'l'],
    &['t', 'x', 't'],
    &['i', 'o'],
    &['c', 'o'],
    &['u', 's'],
    &['u', 'k'],
    &['d', 'e'],
    &['c', 'a'],
    &['a', 'u'],
    &['j', 'p'],
];

fn ends_with_common_tld(input: &[char]) -> bool {
    for tld in COMMON_TLDS {
        let n = tld.len();
        if input.len() >= n && &input[input.len() - n..] == *tld {
            return true;
        }
    }
    false
}

#[cfg(test)]
pub mod tests {
    use super::lex_hostname;

    pub fn example_domain_parts() -> impl Iterator<Item = Vec<char>> {
        [
            r"example.com",
            r"example.com",
            r"example.com",
            r"and.subdomains.example.com",
            r"example.com",
            r"example.com",
            r"example",
            r"s.example",
            r"example.org",
            r"example.org",
            r"example.org",
            r"strange.example.com",
            r"example.org",
            r"example.org",
        ]
        .into_iter()
        .map(|s| s.chars().collect())
    }

    #[test]
    fn can_parse_example_hostnames() {
        for domain in example_domain_parts() {
            dbg!(domain.iter().collect::<String>());
            assert_eq!(lex_hostname(&domain), Some(domain.len()));
        }
    }

    #[test]
    fn hyphen_cannot_open_hostname() {
        let host: Vec<_> = "-something.com".chars().collect();
        assert!(lex_hostname(&host).is_none())
    }
}
