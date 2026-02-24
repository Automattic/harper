mod over_theyre_to_there;
mod typographic_theyre_to_their;

use crate::char_ext::CharExt;
use crate::char_string::CharStringExt;
use crate::Token;

use super::merge_linters::merge_linters;
use over_theyre_to_there::OverTheyreToThere;
use typographic_theyre_to_their::TypographicTheyreToTheir;

fn token_is_theyre(token: &Token, source: &[char]) -> bool {
    if !token.kind.is_word() {
        return false;
    }

    token
        .span
        .get_content(source)
        .normalized()
        .eq_ignore_ascii_case_str("they're")
}

fn token_is_typographic_theyre(token: &Token, source: &[char]) -> bool {
    if !token.kind.is_word() {
        return false;
    }

    let content = token.span.get_content(source);
    content.iter().any(|c| c.normalized() != *c)
        && content.normalized().eq_ignore_ascii_case_str("they're")
}

fn token_is_likely_their_possession(token: &Token, source: &[char]) -> bool {
    if !token.kind.is_word() {
        return false;
    }

    let normalized = token.span.get_content(source).normalized();
    matches!(
        normalized.as_ref(),
        ['b', 'a', 'c', 'k', 'p', 'a', 'c', 'k', 's']
            | ['p', 'a', 't', 'i', 'e', 'n', 'c', 'e']
            | ['d', 'o', 'g']
            | ['p', 'r', 'o', 'p', 'o', 's', 'a', 'l']
            | ['l', 'a', 'u', 'g', 'h', 't', 'e', 'r']
            | ['l', 'a', 'd', 'd', 'e', 'r']
            | ['a', 'p', 'a', 'r', 't', 'm', 'e', 'n', 't']
            | ['m', 'i', 't', 't', 'e', 'n', 's']
            | ['a', 'n', 's', 'w', 'e', 'r']
            | ['s', 'k', 'e', 't', 'c', 'h', 'e', 's']
            | ['s', 'e', 'r', 'v', 'e', 'r']
            | ['b', 'a', 'c', 'k', 'u', 'p']
            | ['e', 'v', 'i', 'd', 'e', 'n', 'c', 'e']
            | ['g', 'a', 'r', 'd', 'e', 'n']
            | ['m', 'a', 'p', 's']
            | ['t', 'e', 'a', 'm']
            | ['p', 'a', 's', 't']
            | ['n', 'e', 'e', 'd', 's']
            | ['p', 'a', 'w', 'n']
            | ['a', 'b', 'i', 'l', 'i', 't', 'y']
            | ['r', 'e', 't', 'u', 'r', 'n']
            | ['h', 'e', 'a', 'r', 'i', 'n', 'g']
            | ['h', 'o', 'u', 's', 'e']
            | ['c', 'o', 'a', 't', 's']
            | ['p', 'r', 'o', 'b', 'l', 'e', 'm', 's']
            | ['u', 'n', 'd', 'e', 'r', 's', 't', 'a', 'n', 'd', 'i', 'n', 'g']
            | ['n', 'e', 'w']
    )
}

merge_linters!(
    TheyreConfusions => OverTheyreToThere, TypographicTheyreToTheir =>
    "Detects apostrophe and locative edge cases that are awkward to model with standard contraction checks."
);
