use std::borrow::Cow;

use smallvec::SmallVec;

/// A char sequence that improves cache locality.
/// Most English words are fewer than 12 characters.
pub type CharString = SmallVec<[char; 16]>;

/// Extensions to character sequences that make them easier to wrangle.
pub trait CharStringExt {
    /// Convert all characters to lowercase, returning a new owned vector if any changes were made.
    fn to_lower(&self) -> Cow<'_, [char]>;

    /// Normalize the character sequence according to the dictionary's standard character set.
    fn normalized(&self) -> Cow<'_, [char]>;

    /// Convert the character sequence to a String.
    fn to_string(&self) -> String;

    /// Case-insensitive comparison with a character slice, assuming the right-hand side is lowercase ASCII.
    /// Only normalizes the left side to lowercase and avoids allocations.
    fn eq_ignore_ascii_case_chars(&self, other: &[char]) -> bool;

    /// Case-insensitive comparison with a string slice, assuming the right-hand side is lowercase ASCII.
    /// Only normalizes the left side to lowercase and avoids allocations.
    fn eq_ignore_ascii_case_str(&self, other: &str) -> bool;

    /// Case-insensitive check if the string ends with the given ASCII suffix.
    /// The suffix is assumed to be lowercase.
    fn ends_with_ignore_ascii_case_chars(&self, suffix: &[char]) -> bool;

    /// Case-insensitive check if the string ends with the given ASCII suffix.
    /// The suffix is assumed to be lowercase.
    fn ends_with_ignore_ascii_case_str(&self, suffix: &str) -> bool;
}

impl CharStringExt for [char] {
    fn to_lower(&self) -> Cow<'_, [char]> {
        if self.iter().all(|c| c.is_lowercase()) {
            return Cow::Borrowed(self);
        }

        let mut out = CharString::with_capacity(self.len());

        out.extend(self.iter().flat_map(|v| v.to_lowercase()));

        Cow::Owned(out.to_vec())
    }

    fn to_string(&self) -> String {
        self.iter().collect()
    }

    /// Convert a given character sequence to the standard character set
    /// the dictionary is in.
    fn normalized(&self) -> Cow<'_, [char]> {
        if self.as_ref().iter().any(|c| char_to_normalized(*c) != *c) {
            Cow::Owned(
                self.as_ref()
                    .iter()
                    .copied()
                    .map(char_to_normalized)
                    .collect(),
            )
        } else {
            Cow::Borrowed(self)
        }
    }

    fn eq_ignore_ascii_case_str(&self, other: &str) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other.chars())
                .all(|(a, b)| a.to_ascii_lowercase() == b)
    }

    fn eq_ignore_ascii_case_chars(&self, other: &[char]) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other.iter())
                .all(|(a, b)| a.to_ascii_lowercase() == *b)
    }

    fn ends_with_ignore_ascii_case_str(&self, suffix: &str) -> bool {
        let suffix_len = suffix.len();
        if self.len() < suffix_len {
            return false;
        }
        self.iter()
            .rev()
            .take(suffix_len)
            .rev()
            .zip(suffix.chars())
            .all(|(a, b)| a.to_ascii_lowercase() == b)
    }

    fn ends_with_ignore_ascii_case_chars(&self, suffix: &[char]) -> bool {
        let suffix_len = suffix.len();
        if self.len() < suffix_len {
            return false;
        }
        self.iter()
            .rev()
            .take(suffix_len)
            .rev()
            .zip(suffix.iter())
            .all(|(a, b)| a.to_ascii_lowercase() == *b)
    }
}

fn char_to_normalized(c: char) -> char {
    match c {
        '’' => '\'',
        '‘' => '\'',
        '＇' => '\'',
        _ => c,
    }
}

macro_rules! char_string {
    ($string:literal) => {{
        use crate::char_string::CharString;

        $string.chars().collect::<CharString>()
    }};
}

pub(crate) use char_string;

#[cfg(test)]
mod tests {
    use super::CharStringExt;

    #[test]
    fn eq_ignore_ascii_case_chars_matches_lowercase() {
        assert!(['H', 'e', 'l', 'l', 'o'].eq_ignore_ascii_case_chars(&['h', 'e', 'l', 'l', 'o']));
    }

    #[test]
    fn eq_ignore_ascii_case_chars_does_not_match_different_word() {
        assert!(!['H', 'e', 'l', 'l', 'o'].eq_ignore_ascii_case_chars(&['w', 'o', 'r', 'l', 'd']));
    }

    #[test]
    fn eq_ignore_ascii_case_str_matches_lowercase() {
        assert!(['H', 'e', 'l', 'l', 'o'].eq_ignore_ascii_case_str("hello"));
    }

    #[test]
    fn eq_ignore_ascii_case_str_does_not_match_different_word() {
        assert!(!['H', 'e', 'l', 'l', 'o'].eq_ignore_ascii_case_str("world"));
    }

    #[test]
    fn ends_with_ignore_ascii_case_chars_matches_suffix() {
        assert!(['H', 'e', 'l', 'l', 'o'].ends_with_ignore_ascii_case_chars(&['l', 'o']));
    }

    #[test]
    fn ends_with_ignore_ascii_case_chars_does_not_match_different_suffix() {
        assert!(
            !['H', 'e', 'l', 'l', 'o']
                .ends_with_ignore_ascii_case_chars(&['w', 'o', 'r', 'l', 'd'])
        );
    }

    #[test]
    fn ends_with_ignore_ascii_case_str_matches_suffix() {
        assert!(['H', 'e', 'l', 'l', 'o'].ends_with_ignore_ascii_case_str("lo"));
    }

    #[test]
    fn ends_with_ignore_ascii_case_str_does_not_match_different_suffix() {
        assert!(!['H', 'e', 'l', 'l', 'o'].ends_with_ignore_ascii_case_str("world"));
    }
}
