use crate::char_ext::CharExt;
use serde::{Deserialize, Serialize};

/// Orthography information.
pub enum Orthography {
    /// Every char that is a letter is lowercase.
    Lowercase = 1 << 0,
    /// First char is uppercase, the rest is lowercase (but multi-word?)
    Titlecase = 1 << 1,
    /// Every char that is a letter is uppercase (including single-letter uppercase)
    AllCaps = 1 << 2,
    /// Starts with a lowercase letter but also contains uppercase letters.
    LowerCamel = 1 << 3,
    /// Starts with an uppercase letter but also contains lowercase letters. (Superset of Titlecase.)
    UpperCamel = 1 << 4,
    /// Contains at least one space.
    Multiword = 1 << 5,
    /// Contains at least one hyphen.
    Hyphenated = 1 << 6,
    /// Contains an apostrophe, so it's a possessive or a contraction.
    Apostrophe = 1 << 7,
    /// Could be Roman numerals.
    RomanNumerals = 1 << 8,
}

/// The underlying type used for OrthographyFlags.
type OrthographyFlagsUnderlyingType = u16;

bitflags::bitflags! {
    /// A collection of bit flags used to represent orthographic properties of a word.
    ///
    /// This is generally used to allow a word (or similar) to be tagged with multiple orthographic
    /// properties.
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Serialize)]
    pub struct OrthFlags: OrthographyFlagsUnderlyingType {
        const LOWERCASE = Orthography::Lowercase as OrthographyFlagsUnderlyingType;
        const TITLECASE = Orthography::Titlecase as OrthographyFlagsUnderlyingType;
        const ALLCAPS = Orthography::AllCaps as OrthographyFlagsUnderlyingType;
        const LOWER_CAMEL = Orthography::LowerCamel as OrthographyFlagsUnderlyingType;
        const UPPER_CAMEL = Orthography::UpperCamel as OrthographyFlagsUnderlyingType;
        const MULTIWORD = Orthography::Multiword as OrthographyFlagsUnderlyingType;
        const HYPHENATED = Orthography::Hyphenated as OrthographyFlagsUnderlyingType;
        const APOSTROPHE = Orthography::Apostrophe as OrthographyFlagsUnderlyingType;
        const ROMAN_NUMERALS = Orthography::RomanNumerals as OrthographyFlagsUnderlyingType;
    }
}
impl Default for OrthFlags {
    fn default() -> Self {
        Self::empty()
    }
}

impl OrthFlags {
    /// An `OrthFlags` with all casing-related flags set.
    pub const CASE_FLAGS: OrthFlags = OrthFlags::LOWERCASE
        .union(OrthFlags::TITLECASE)
        .union(OrthFlags::ALLCAPS)
        .union(OrthFlags::LOWER_CAMEL)
        .union(OrthFlags::UPPER_CAMEL);

    /// Mask out all flags not related to casing.
    pub const fn case_flags(self) -> Self {
        self.intersection(Self::CASE_FLAGS)
    }

    /// Construct orthography flags for a given sequence of letters.
    pub fn from_letters(letters: impl IntoIterator<Item = char>) -> Self {
        let mut ortho_flags = Self::default();
        let mut all_lower = true;
        let mut all_upper = true;
        let mut first_is_upper = false;
        let mut first_is_lower = false;
        let mut saw_upper_after_first = false;
        let mut saw_lower_after_first = false;
        let mut is_first_char = true;
        let mut letter_count = 0usize;

        // Track Roman numeral candidacy in a single pass without heap allocation.
        // Longest standard Roman numeral is 15 chars ("MMMDCCCLXXXVIII"), 16 handles up to 4 M's.
        let mut roman_buf = ['\0'; 16];
        let mut roman_len = 0usize;
        let mut roman_first_upper: Option<bool> = None;
        let mut could_be_roman = true;

        for c in letters {
            if could_be_roman {
                let lower = c.to_ascii_lowercase();
                let is_upper = c.is_uppercase();
                let same_case = match roman_first_upper {
                    Some(first_upper) => is_upper == first_upper,
                    None => {
                        roman_first_upper = Some(is_upper);
                        true
                    }
                };

                if same_case
                    && matches!(lower, 'm' | 'd' | 'c' | 'l' | 'x' | 'v' | 'i')
                    && roman_len < roman_buf.len()
                {
                    roman_buf[roman_len] = lower;
                    roman_len += 1;
                } else {
                    could_be_roman = false;
                }
            }

            if c == ' ' {
                ortho_flags |= Self::MULTIWORD;
                continue;
            }

            if c == '-' {
                ortho_flags |= Self::HYPHENATED;
                continue;
            }

            if c.normalized() == '\'' {
                ortho_flags |= Self::APOSTROPHE;
                continue;
            }

            if !c.is_english_lingual() {
                continue;
            }

            letter_count += 1;

            if c.is_lowercase() {
                all_upper = false;
                if is_first_char {
                    first_is_lower = true;
                } else {
                    saw_lower_after_first = true;
                }
            } else if c.is_uppercase() {
                all_lower = false;
                if is_first_char {
                    first_is_upper = true;
                } else {
                    saw_upper_after_first = true;
                }
            } else {
                first_is_upper = false;
                first_is_lower = false;
            }
            is_first_char = false;
        }

        if letter_count > 0 {
            if all_lower {
                ortho_flags |= Self::LOWERCASE;
            }
            if all_upper {
                ortho_flags |= Self::ALLCAPS;
            }
            if letter_count > 1 && first_is_upper && !saw_upper_after_first {
                ortho_flags |= Self::TITLECASE;
            }
            if first_is_lower && saw_upper_after_first {
                ortho_flags |= Self::LOWER_CAMEL;
            }
            if first_is_upper && saw_lower_after_first && saw_upper_after_first {
                ortho_flags |= Self::UPPER_CAMEL;
            }
        }

        if could_be_roman
            && roman_len > 0
            && is_really_roman_numerals(roman_buf[..roman_len].iter().copied())
        {
            ortho_flags |= Self::ROMAN_NUMERALS;
        }

        ortho_flags
    }
}

fn is_really_roman_numerals(word: impl IntoIterator<Item = char>) -> bool {
    let mut chars = word.into_iter().peekable();

    let mut m_count = 0;
    while m_count < 4 && chars.peek() == Some(&'m') {
        chars.next();
        m_count += 1;
    }

    if !check_roman_group(&mut chars, 'c', 'd', 'm') {
        return false;
    }

    if !check_roman_group(&mut chars, 'x', 'l', 'c') {
        return false;
    }

    if !check_roman_group(&mut chars, 'i', 'v', 'x') {
        return false;
    }

    chars.next().is_none()
}

fn check_roman_group<I: Iterator<Item = char>>(
    chars: &mut std::iter::Peekable<I>,
    one: char,
    five: char,
    ten: char,
) -> bool {
    match chars.peek() {
        Some(&c) if c == one => {
            chars.next();
            match chars.peek() {
                Some(&next) if next == ten || next == five => {
                    chars.next();
                    true
                }
                _ => {
                    let mut count = 0;
                    while count < 2 && chars.peek() == Some(&one) {
                        chars.next();
                        count += 1;
                    }
                    true
                }
            }
        }
        Some(&c) if c == five => {
            chars.next();
            let mut count = 0;
            while count < 3 && chars.peek() == Some(&one) {
                chars.next();
                count += 1;
            }
            true
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use crate::CharString;
    use crate::dict_word_metadata::tests::md;
    use crate::dict_word_metadata_orthography::OrthFlags;

    fn orth_flags(s: &str) -> OrthFlags {
        let letters: CharString = s.chars().collect();
        OrthFlags::from_letters(letters)
    }

    #[test]
    fn test_lowercase_flags() {
        let flags = orth_flags("hello");
        assert!(flags.contains(OrthFlags::LOWERCASE));
        assert!(!flags.contains(OrthFlags::TITLECASE));
        assert!(!flags.contains(OrthFlags::ALLCAPS));
        assert!(!flags.contains(OrthFlags::LOWER_CAMEL));
        assert!(!flags.contains(OrthFlags::UPPER_CAMEL));

        let flags = orth_flags("hello123");
        assert!(flags.contains(OrthFlags::LOWERCASE));
    }

    #[test]
    fn test_titlecase_flags() {
        let flags = orth_flags("Hello");
        assert!(!flags.contains(OrthFlags::LOWERCASE));
        assert!(flags.contains(OrthFlags::TITLECASE));
        assert!(!flags.contains(OrthFlags::ALLCAPS));
        assert!(!flags.contains(OrthFlags::LOWER_CAMEL));
        assert!(!flags.contains(OrthFlags::UPPER_CAMEL));

        assert!(orth_flags("World").contains(OrthFlags::TITLECASE));
        assert!(orth_flags("Something").contains(OrthFlags::TITLECASE));
        assert!(!orth_flags("McDonald").contains(OrthFlags::TITLECASE));
        assert!(!orth_flags("O'Reilly").contains(OrthFlags::TITLECASE));
        assert!(!orth_flags("A").contains(OrthFlags::TITLECASE));
    }

    #[test]
    fn test_allcaps_flags() {
        let flags = orth_flags("HELLO");
        assert!(!flags.contains(OrthFlags::LOWERCASE));
        assert!(!flags.contains(OrthFlags::TITLECASE));
        assert!(flags.contains(OrthFlags::ALLCAPS));
        assert!(!flags.contains(OrthFlags::LOWER_CAMEL));
        assert!(!flags.contains(OrthFlags::UPPER_CAMEL));

        assert!(orth_flags("NASA").contains(OrthFlags::ALLCAPS));
        assert!(orth_flags("I").contains(OrthFlags::ALLCAPS));
    }

    #[test]
    fn test_lower_camel_flags() {
        let flags = orth_flags("helloWorld");
        assert!(!flags.contains(OrthFlags::LOWERCASE));
        assert!(!flags.contains(OrthFlags::TITLECASE));
        assert!(!flags.contains(OrthFlags::ALLCAPS));
        assert!(flags.contains(OrthFlags::LOWER_CAMEL));
        assert!(!flags.contains(OrthFlags::UPPER_CAMEL));

        assert!(orth_flags("getHTTPResponse").contains(OrthFlags::LOWER_CAMEL));
        assert!(orth_flags("eBay").contains(OrthFlags::LOWER_CAMEL));
        assert!(!orth_flags("hello").contains(OrthFlags::LOWER_CAMEL));
        assert!(!orth_flags("HelloWorld").contains(OrthFlags::LOWER_CAMEL));
    }

    #[test]
    fn test_upper_camel_flags() {
        let flags = orth_flags("HelloWorld");
        assert!(!flags.contains(OrthFlags::LOWERCASE));
        assert!(!flags.contains(OrthFlags::TITLECASE));
        assert!(!flags.contains(OrthFlags::ALLCAPS));
        assert!(!flags.contains(OrthFlags::LOWER_CAMEL));
        assert!(flags.contains(OrthFlags::UPPER_CAMEL));

        assert!(orth_flags("HttpRequest").contains(OrthFlags::UPPER_CAMEL));
        assert!(orth_flags("McDonald").contains(OrthFlags::UPPER_CAMEL));
        assert!(orth_flags("O'Reilly").contains(OrthFlags::UPPER_CAMEL));
        assert!(orth_flags("XMLHttpRequest").contains(OrthFlags::UPPER_CAMEL));
        assert!(!orth_flags("Hello").contains(OrthFlags::UPPER_CAMEL));
        assert!(!orth_flags("NASA").contains(OrthFlags::UPPER_CAMEL));
        assert!(!orth_flags("Hi").contains(OrthFlags::UPPER_CAMEL));
    }

    #[test]
    fn test_roman_numeral_flags() {
        assert!(orth_flags("MCMXCIV").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(orth_flags("mdccclxxi").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(orth_flags("MMXXI").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(orth_flags("mcmxciv").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(orth_flags("MCMXCIV").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(orth_flags("MMI").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(orth_flags("MMXXV").contains(OrthFlags::ROMAN_NUMERALS));
    }

    #[test]
    fn test_single_roman_numeral_flags() {
        assert!(orth_flags("i").contains(OrthFlags::ROMAN_NUMERALS));
    }

    #[test]
    fn empty_string_is_not_roman_numeral() {
        assert!(!orth_flags("").contains(OrthFlags::ROMAN_NUMERALS));
    }

    #[test]
    fn dont_allow_mixed_case_roman_numerals() {
        assert!(!orth_flags("MCMlxxxVIII").contains(OrthFlags::ROMAN_NUMERALS));
    }

    #[test]
    fn dont_allow_looks_like_but_isnt_roman_numeral() {
        assert!(!orth_flags("mdxlivx").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(!orth_flags("XIXIVV").contains(OrthFlags::ROMAN_NUMERALS));
    }

    #[test]
    fn australia_lexeme_is_titlecase_even_when_word_is_lowercase() {
        assert!(md("australia").orth_info.contains(OrthFlags::TITLECASE));
    }

    #[test]
    fn australia_lexeme_is_titlecase_even_when_word_is_all_caps() {
        assert!(md("AUSTRALIA").orth_info.contains(OrthFlags::TITLECASE));
    }

    #[test]
    fn australia_lexeme_is_titlecase_even_when_word_is_mixed_case() {
        assert!(md("AuStrAlIA").orth_info.contains(OrthFlags::TITLECASE));
    }

    #[test]
    fn db_and_kw_symbols_are_lower_camel_case() {
        // dB, kW
        assert!(md("db").orth_info.contains(OrthFlags::LOWER_CAMEL));
    }

    #[test]
    fn am_is_lowercase_and_titlecase_and_all_caps() {
        // am, Am, AM
        let metadata = md("am");
        assert!(metadata.orth_info.contains(OrthFlags::LOWERCASE));
        assert!(metadata.orth_info.contains(OrthFlags::TITLECASE));
        assert!(metadata.orth_info.contains(OrthFlags::ALLCAPS));
    }

    #[test]
    fn reading_is_both_lowercase_and_titlecase() {
        // Reading is a town in England
        let metadata = md("reading");
        assert!(metadata.orth_info.contains(OrthFlags::LOWERCASE));
        assert!(metadata.orth_info.contains(OrthFlags::TITLECASE));
    }

    #[test]
    fn ebay_and_esim_are_lower_camel() {
        // eBay eSIM
        let md1 = md("ebay");
        assert!(md1.orth_info.contains(OrthFlags::LOWER_CAMEL));
        let md2 = md("esim");
        assert!(md2.orth_info.contains(OrthFlags::LOWER_CAMEL));
    }

    #[test]
    fn test_orthflags_titlecase_on_str() {
        let go_str: &str = "Go";
        assert!(orth_flags(go_str).contains(OrthFlags::TITLECASE));
    }

    #[test]
    fn test_orthflags_allcaps_on_string() {
        let go_string: String = "GO".to_string();
        assert!(orth_flags(&go_string).contains(OrthFlags::ALLCAPS));
    }
}
