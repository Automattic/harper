use std::borrow::Cow;

use itertools::Itertools;

use crate::case::Case::Upper;
use crate::char_ext::CharExt;
use crate::{CaseIterExt, Dialect};

#[derive(PartialEq)]
pub enum InitialSound {
    Vowel,
    Consonant,
    Either, // for SQL
}

/// Checks whether a provided word begins with a vowel _sound_. Returns `None` if `word` is empty.
///
/// It was produced through trial and error.
/// Matches with 99.71% and 99.77% of vowels and non-vowels in the
/// Carnegie-Mellon University word -> pronunciation dataset.
pub fn starts_with_vowel(word: &[char], dialect: Dialect) -> Option<InitialSound> {
    if word.is_empty() {
        return None;
    }

    if matches!(word, ['L', 'E', 'D'] | ['S', 'Q', 'L'] | ['U', 'R', 'L']) {
        return Some(InitialSound::Either);
    }

    // Try to get the first chunk of a word that appears to be a partial initialism.
    // For example:
    // - `RFL` from `RFLink`
    // - `m` from `mDNS`
    let word = {
        let word_casing = word.get_casing_unfiltered();
        match word_casing.as_slice() {
            // Lower-upper or upper-upper, possibly a (partial) initialism.
            [Some(first_char_case), Some(Upper), ..] => {
                &word[0..word_casing
                    .iter()
                    .position(|c| *c != Some(*first_char_case))
                    .unwrap_or(word.len())]
            }
            // Lower-lower or upper-lower, unlikely to be a partial initialism.
            _ => word,
        }
    };

    let is_likely_initialism = word.iter().all(|c| !c.is_alphabetic() || c.is_uppercase());

    if word.len() == 1 || (is_likely_initialism && !is_likely_acronym(word)) {
        return Some(
            if matches!(
                word[0].to_ascii_uppercase(),
                'A' | 'E' | 'F' | 'H' | 'I' | 'L' | 'M' | 'N' | 'O' | 'R' | 'S' | 'X'
            ) {
                InitialSound::Vowel
            } else {
                InitialSound::Consonant
            },
        );
    }

    let word = to_lower_word(word);
    let word = word.as_ref();

    if matches!(word, ['u', 'b', 'i', ..]) {
        return Some(InitialSound::Either);
    }

    if matches!(word, ['e', 'u', 'l', 'e', ..]) {
        return Some(InitialSound::Vowel);
    }

    if matches!(
        word,
        ['u', 'k', ..]
            | ['u', 'd', 'e', ..] // for 'udev'
            | ['e', 'u', 'p', 'h', ..]
            | ['e', 'u', 'g' | 'l' | 'c', ..]
            | ['o', 'n', 'e', ..]
            | ['o', 'n', 'c', 'e']
    ) {
        return Some(InitialSound::Consonant);
    }

    if matches!(
        word,
        ['h', 'o', 'u', 'r', ..]
            | ['u', 'n', 'i', 'n' | 'm', ..]
            | ['u', 'n', 'a' | 'u', ..]
            | ['u', 'r', 'b', ..]
            | ['i', 'n', 't', ..]
    ) {
        return Some(InitialSound::Vowel);
    }

    if matches!(word, ['h', 'e', 'r', 'b', ..] if dialect == Dialect::American || dialect == Dialect::Canadian)
    {
        return Some(InitialSound::Vowel);
    }

    if matches!(word, ['u', 'n' | 's', 'i' | 'a' | 'u', ..]) {
        return Some(InitialSound::Consonant);
    }

    if matches!(word, ['u', 'n', ..]) {
        return Some(InitialSound::Vowel);
    }

    if matches!(word, ['u', 'r', 'g', ..]) {
        return Some(InitialSound::Vowel);
    }

    if matches!(word, ['u', 't', 't', ..]) {
        return Some(InitialSound::Vowel);
    }

    if matches!(
        word,
        ['u', 't' | 'r' | 'n', ..] | ['e', 'u', 'r', ..] | ['u', 'w', ..] | ['u', 's', 'e', ..]
    ) {
        return Some(InitialSound::Consonant);
    }

    if matches!(word, ['o', 'n', 'e', 'a' | 'e' | 'i' | 'u', 'l' | 'd', ..]) {
        return Some(InitialSound::Vowel);
    }

    if matches!(word, ['o', 'n', 'e', 'a' | 'e' | 'i' | 'u' | '-' | 's', ..]) {
        return Some(InitialSound::Consonant);
    }

    if matches!(
        word,
        ['s', 'o', 's']
            | ['r', 'z', ..]
            | ['n', 'g', ..]
            | ['n', 'v', ..]
            | ['x', 'b', 'o', 'x']
            | ['h', 'e', 'i', 'r', ..]
            | ['h', 'o', 'n', 'o', 'r', ..]
            | ['h', 'o', 'n', 'e', 's', ..]
    ) {
        return Some(InitialSound::Vowel);
    }

    if matches!(
        word,
        ['j', 'u' | 'o', 'n', ..] | ['j', 'u', 'r', 'a' | 'i' | 'o', ..]
    ) {
        return Some(InitialSound::Consonant);
    }

    if matches!(word, ['x', '-' | '\'' | '.' | 'o' | 's', ..]) {
        return Some(InitialSound::Vowel);
    }

    if word[0].is_vowel() {
        return Some(InitialSound::Vowel);
    }

    Some(InitialSound::Consonant)
}

fn to_lower_word(word: &[char]) -> Cow<'_, [char]> {
    if word.iter().any(|c| c.is_uppercase()) {
        Cow::Owned(
            word.iter()
                .flat_map(|c| c.to_lowercase())
                .collect::<Vec<_>>(),
        )
    } else {
        Cow::Borrowed(word)
    }
}

fn is_likely_acronym(word: &[char]) -> bool {
    /// Does the word contain any sequences that might indicate it's not an acronym?
    fn word_contains_false_positive_sequence(word: &[char]) -> bool {
        let likely_false_positive_sequences = [['V', 'C']];
        for fp_sequence in likely_false_positive_sequences {
            if word
                .windows(fp_sequence.len())
                .any(|subslice| subslice == fp_sequence)
            {
                return true;
            }
        }
        false
    }

    // If the initialism is shorter than this, skip it.
    const MIN_LEN: usize = 3;

    if let Some(first_chars) = word.get(..MIN_LEN)
        // Unlikely to be an acronym if it contains non-alphabetic characters.
        && first_chars.iter().copied().all(char::is_alphabetic)
        && !word_contains_false_positive_sequence(word)
    {
        let vowel_map = first_chars
            .iter()
            .map(CharExt::is_vowel)
            .collect_array::<MIN_LEN>()
            .unwrap();
        matches!(vowel_map, [false, true, false] | [false, true, true])
    } else {
        false
    }
}
