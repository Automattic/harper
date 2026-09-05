use super::SingleTokenPattern;

use crate::{CharString, Token};

/// Determines how case matching is performed for a word pattern.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CaseMatchingMode {
    /// Match regardless of case (case insensitive)
    CaseInsensitive,
    /// Match exact case only (case sensitive)
    CaseSensitive,
    /// Match only standard case patterns: all lowercase, all uppercase, or title case
    StandardCase,
}

/// Matches a predefined word.
#[derive(Clone)]
pub struct Word {
    /// The word to match.
    word: CharString,
    /// Determines how case matching is performed.
    case_mode: CaseMatchingMode,
}

impl Word {
    /// Matches the provided word, ignoring case.
    pub fn new(word: &'static str) -> Self {
        Self {
            word: word.chars().collect(),
            case_mode: CaseMatchingMode::CaseInsensitive,
        }
    }

    /// Matches the provided word, ignoring case.
    pub fn from_chars(word: &[char]) -> Self {
        Self {
            word: word.iter().copied().collect(),
            case_mode: CaseMatchingMode::CaseInsensitive,
        }
    }

    /// Matches the provided word, ignoring case.
    pub fn from_char_string(word: CharString) -> Self {
        Self {
            word,
            case_mode: CaseMatchingMode::CaseInsensitive,
        }
    }

    /// Matches the provided word, case-sensitive.
    pub fn new_exact(word: &'static str) -> Self {
        Self {
            word: word.chars().collect(),
            case_mode: CaseMatchingMode::CaseSensitive,
        }
    }

    /// Matches the provided word, but only if it follows standard case patterns:
    /// all lowercase, all uppercase, or title case (first letter uppercase, rest lowercase).
    /// This avoids matching trademarks, app names, and other mixed-case patterns.
    pub fn new_standard_case(word: &'static str) -> Self {
        Self {
            word: word.chars().collect(),
            case_mode: CaseMatchingMode::StandardCase,
        }
    }

    /// Checks if the given characters follow a standard case pattern:
    /// - All lowercase
    /// - All uppercase  
    /// - Title case (first letter uppercase, rest lowercase)
    fn is_standard_case(chars: &[char]) -> bool {
        match chars.len() {
            0 => false,
            1 => true,
            _ => {
                let (first, rest) = chars.split_at(1);
                let c0 = first[0];
                let c0_is_lowercase = c0.is_lowercase();

                let (basis, rest) = if !c0_is_lowercase {
                    (rest[0].is_lowercase(), &rest[1..])
                } else {
                    (c0_is_lowercase, rest)
                };

                rest.iter().all(|c| c.is_lowercase() == basis)
            }
        }
    }
}

impl SingleTokenPattern for Word {
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        if !token.kind.is_word() {
            return false;
        }
        if token.span.len() != self.word.len() {
            return false;
        }

        let chars = token.get_ch(source);

        match self.case_mode {
            CaseMatchingMode::CaseSensitive => chars == self.word.as_slice(),
            _ => {
                // Both CaseInsensitive and StandardCase need case-insensitive matching
                // but StandardCase has an additional filter
                if matches!(self.case_mode, CaseMatchingMode::StandardCase)
                    && !Self::is_standard_case(chars)
                {
                    return false;
                }
                chars
                    .iter()
                    .zip(&self.word)
                    .all(|(a, b)| a.eq_ignore_ascii_case(b))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Document, Span, linting::tests::SpanVecExt, patterns::DocPattern};

    use super::Word;

    #[test]
    fn fruit() {
        let doc = Document::new_plain_english_curated("I ate a banana and an apple today.");

        assert_eq!(
            Word::new("banana").find_all_matches_in_doc(&doc),
            vec![Span::new(6, 7)]
        );
        assert_eq!(
            Word::new_exact("banana").find_all_matches_in_doc(&doc),
            vec![Span::new(6, 7)]
        );
    }

    #[test]
    fn fruit_whack_capitalization() {
        let doc = Document::new_plain_english_curated("I Ate A bAnaNa And aN apPlE today.");

        assert_eq!(
            Word::new("banana").find_all_matches_in_doc(&doc),
            vec![Span::new(6, 7)]
        );
        assert_eq!(
            Word::new_exact("banana").find_all_matches_in_doc(&doc),
            vec![]
        );
    }

    #[test]
    fn standard_case_basic_matches() {
        let doc =
            Document::new_plain_english_curated("I ate a banana and BANANA and Banana today.");

        // Should match all three standard case variations
        let matches = Word::new_standard_case("banana").find_all_matches_in_doc(&doc);
        assert_eq!(matches.to_strings(&doc), vec!["banana", "BANANA", "Banana"]);
    }

    #[test]
    fn standard_case_rejects_mixed_case() {
        let doc = Document::new_plain_english_curated("I saw iPhone and iPad and YouTube today.");

        // Should not match any mixed-case words
        let iphone_matches: Vec<String> = Word::new_standard_case("iphone")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);
        let ipad_matches: Vec<String> = Word::new_standard_case("ipad")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);
        let youtube_matches: Vec<String> = Word::new_standard_case("youtube")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);

        assert_eq!(iphone_matches, Vec::<String>::new());
        assert_eq!(ipad_matches, Vec::<String>::new());
        assert_eq!(youtube_matches, Vec::<String>::new());
    }

    #[test]
    fn standard_case_rejects_pascal_and_camel_case() {
        let doc = Document::new_plain_english_curated(
            "I saw BananaTree and bananaTree and BaNaNa today.",
        );

        // Should not match PascalCase or camelCase
        let bananatree_matches: Vec<String> = Word::new_standard_case("bananatree")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);
        let banana_matches: Vec<String> = Word::new_standard_case("banana")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);

        assert_eq!(bananatree_matches, Vec::<String>::new());
        assert_eq!(banana_matches, Vec::<String>::new());
    }

    #[test]
    fn standard_case_single_letters() {
        let doc = Document::new_plain_english_curated("A B C a b c I i");

        // Should match single letters in both cases
        let a_matches: Vec<String> = Word::new_standard_case("a")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);
        let b_matches: Vec<String> = Word::new_standard_case("b")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);
        let i_matches: Vec<String> = Word::new_standard_case("i")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);

        assert_eq!(a_matches, vec!["A", "a"]);
        assert_eq!(b_matches, vec!["B", "b"]);
        assert_eq!(i_matches, vec!["I", "i"]);
    }

    #[test]
    fn standard_case_vs_exact_case() {
        let doc = Document::new_plain_english_curated("I ate banana and BANANA and Banana.");

        // new_exact should only match exact case
        let exact_matches: Vec<String> = Word::new_exact("banana")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);
        assert_eq!(exact_matches, vec!["banana"]);

        // new_standard_case should match all standard case variations
        let standard_matches: Vec<String> = Word::new_standard_case("banana")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);
        assert_eq!(standard_matches, vec!["banana", "BANANA", "Banana"]);
    }

    #[test]
    fn standard_case_edge_cases() {
        let doc = Document::new_plain_english_curated("A a B b I i.");

        // Test single characters and common pronouns
        let a_matches: Vec<String> = Word::new_standard_case("a")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);
        let i_matches: Vec<String> = Word::new_standard_case("i")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);

        assert_eq!(a_matches, vec!["A", "a"]);
        assert_eq!(i_matches, vec!["I", "i"]);
    }

    #[test]
    fn standard_case_complex_examples() {
        let doc = Document::new_plain_english_curated(
            "The iPhone is made by Apple but the apple is fruit.",
        );

        // Should match both "Apple" and "apple" (both are standard case)
        let apple_matches: Vec<String> = Word::new_standard_case("apple")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);
        assert_eq!(apple_matches, vec!["Apple", "apple"]);

        // Should not match iPhone (not standard case)
        let iphone_matches: Vec<String> = Word::new_standard_case("iphone")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);
        assert_eq!(iphone_matches, Vec::<String>::new());
    }

    #[test]
    fn match_all_standard_3_letter_sets() {
        let doc = Document::new_plain_english_curated("abc abC aBc aBC Abc AbC ABc ABC");

        let matches: Vec<String> = Word::new_standard_case("abc")
            .find_all_matches_in_doc(&doc)
            .to_strings(&doc);

        assert_eq!(matches, vec!["abc", "Abc", "ABC"]);
    }
}
