use crate::CharStringExt;

/// Checks whether a provided word begins with a vowel _sound_.
///
/// It was produced through trial and error.
/// Matches with 99.71% and 99.77% of vowels and non-vowels in the
/// Carnegie-Mellon University word -> pronunciation dataset.
pub fn starts_with_vowel(word: &[char]) -> bool {
    let is_likely_initialism = word.iter().all(|c| c.is_uppercase());

    if is_likely_initialism && !word.is_empty() {
        return matches!(
            word[0],
            'A' | 'E' | 'F' | 'H' | 'I' | 'L' | 'M' | 'N' | 'O' | 'R' | 'S' | 'X'
        );
    }

    let word = word.to_lower();
    let word = word.as_ref();

    if matches!(
        word,
        [] | ['u', 'k', ..]
            | ['e', 'u', 'p', 'h', ..]
            | ['e', 'u', 'g' | 'l' | 'c', ..]
            | ['o', 'n', 'e']
            | ['o', 'n', 'c', 'e']
    ) {
        return false;
    }

    if matches!(word, |['h', 'o', 'u', 'r', ..]| ['h', 'o', 'n', ..]
        | ['u', 'n', 'i', 'n' | 'm', ..]
        | ['u', 'n', 'a' | 'u', ..]
        | ['h', 'e', 'r', 'b', ..]
        | ['u', 'r', 'b', ..]
        | ['i', 'n', 't', ..])
    {
        return true;
    }

    if matches!(word, ['u', 'n' | 's', 'i' | 'a' | 'u', ..]) {
        return false;
    }

    if matches!(word, ['u', 'n', ..]) {
        return true;
    }

    if matches!(word, ['u', 'r', 'g', ..]) {
        return true;
    }

    if matches!(word, ['u', 't', 't', ..]) {
        return true;
    }

    if matches!(
        word,
        ['u', 't' | 'r' | 'n', ..] | ['e', 'u', 'r', ..] | ['u', 'w', ..] | ['u', 's', 'e', ..]
    ) {
        return false;
    }

    if matches!(word, ['o', 'n', 'e', 'a' | 'e' | 'i' | 'u', 'l' | 'd', ..]) {
        return true;
    }

    if matches!(word, ['o', 'n', 'e', 'a' | 'e' | 'i' | 'u' | '-' | 's', ..]) {
        return false;
    }

    if matches!(
        word,
        ['s', 'o', 's']
            | ['r', 'z', ..]
            | ['n', 'g', ..]
            | ['n', 'v', ..]
            | ['x']
            | ['x', 'b', 'o', 'x']
            | ['h', 'e', 'i', 'r', ..]
            | ['h', 'o', 'n', 'o', 'r', ..]
    ) {
        return true;
    }

    if matches!(
        word,
        ['j', 'u' | 'o', 'n', ..] | ['j', 'u', 'r', 'a' | 'i' | 'o', ..]
    ) {
        return false;
    }

    if matches!(word, ['x', '-' | '\'' | '.' | 'o' | 's', ..]) {
        return true;
    }

    matches!(
        word,
        ['a', ..] | ['e', ..] | ['i', ..] | ['o', ..] | ['u', ..]
    )
}
