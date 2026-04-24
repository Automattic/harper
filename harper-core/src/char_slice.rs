pub struct CaseInsensitiveCharSlice<'a>(&'a [char]);

impl<'a> CaseInsensitiveCharSlice<'a> {
    pub fn new(slice: &'a [char]) -> Self {
        CaseInsensitiveCharSlice(slice)
    }

    /// Get the underlying character slice.
    pub fn as_slice(&self) -> &'a [char] {
        self.0
    }
}

/// Case-insensitive comparison with a character slice, assuming the right-hand side is lowercase ASCII.
/// Only normalizes the left side to lowercase and avoids allocations.
impl PartialEq<&[char]> for CaseInsensitiveCharSlice<'_> {
    fn eq(&self, other: &&[char]) -> bool {
        debug_assert!(
            other
                .iter()
                .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_lowercase()),
            "Right-hand side character slice contains non-lowercase ASCII characters"
        );

        self.0
            .iter()
            .map(char::to_ascii_lowercase)
            .eq(other.iter().copied())
    }
}

/// Case-insensitive comparison with a character array, assuming the right-hand side is lowercase ASCII.
/// Only normalizes the left side to lowercase and avoids allocations.
impl<const N: usize> PartialEq<&[char; N]> for CaseInsensitiveCharSlice<'_> {
    fn eq(&self, other: &&[char; N]) -> bool {
        *self == &other[..]
    }
}

/// Case-insensitive comparison with a character array, assuming the right-hand side is lowercase ASCII.
/// Only normalizes the left side to lowercase and avoids allocations.
impl<const N: usize> PartialEq<[char; N]> for CaseInsensitiveCharSlice<'_> {
    fn eq(&self, other: &[char; N]) -> bool {
        *self == &other[..]
    }
}

/// Case-insensitive comparison with a string slice, assuming the right-hand side is lowercase ASCII.
/// Only normalizes the left side to lowercase and avoids allocations.
impl PartialEq<&str> for CaseInsensitiveCharSlice<'_> {
    fn eq(&self, other: &&str) -> bool {
        debug_assert!(
            other
                .chars()
                .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_lowercase()),
            "Right-hand side string contains non-lowercase ASCII characters"
        );

        let chit = self.0.iter();
        let strit = other.chars();

        chit.map(char::to_ascii_lowercase).eq(strit)
    }
}

// TODO I can't get this one to work.
// TODO It would be used in `harper-core/src/linting/plural_wrong_word_of_phrase.rs`
// impl PartialEq<String> for CaseInsensitiveCharSlice<'_> {
//     fn eq(&self, other: &String) -> bool {
//         self == other.as_str()
//     }
// }

impl PartialEq<str> for &CaseInsensitiveCharSlice<'_> {
    fn eq(&self, other: &str) -> bool {
        **self == other
    }
}

/// Case-insensitive comparison with a character slice, assuming the right-hand side is lowercase ASCII.
/// Only normalizes the left side to lowercase and avoids allocations.
impl PartialEq<&[char]> for &CaseInsensitiveCharSlice<'_> {
    fn eq(&self, other: &&[char]) -> bool {
        debug_assert!(
            other
                .iter()
                .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_lowercase()),
            "Right-hand side character slice contains non-lowercase ASCII characters"
        );

        **self == *other
    }
}

/// Case-insensitive comparison with a character array, assuming the right-hand side is lowercase ASCII.
/// Only normalizes the left side to lowercase and avoids allocations.
impl<const N: usize> PartialEq<[char; N]> for &CaseInsensitiveCharSlice<'_> {
    fn eq(&self, other: &[char; N]) -> bool {
        **self == &other[..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Right-hand side string contains non-lowercase ASCII characters")]
    fn debug_asserts_non_lowercase_string() {
        let slice = CaseInsensitiveCharSlice::new(&['h', 'e', 'l', 'l', 'o']);
        let _ = slice == "World"; // Contains uppercase 'W'
    }

    #[test]
    fn debug_asserts_allows_lowercase_string() {
        let slice = CaseInsensitiveCharSlice::new(&['h', 'e', 'l', 'l', 'o']);
        assert!(slice == "hello"); // All lowercase - should not panic
    }
}
