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
/// At the time of writing, this is currently a `u8`. If we want to define more than 8 orthographic
/// properties in the future, we will need to switch this to a larger type.
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

#[cfg(test)]
mod tests {
    use crate::dict_word_metadata::tests::md;
    use crate::dict_word_metadata_orthography::OrthFlags;

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
}
