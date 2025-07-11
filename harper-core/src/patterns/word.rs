use super::SingleTokenPattern;

use crate::{CharString, Token};

/// Matches a predefined word.
#[derive(Clone)]
pub struct Word {
    /// The word to match.
    word: CharString,
    /// Determines whether the match is case-sensitive.
    case_sensitive: bool,
}

impl Word {
    /// Matches the provided word, ignoring case.
    pub fn new(word: &'static str) -> Self {
        Self {
            word: word.chars().collect(),
            case_sensitive: false,
        }
    }

    /// Matches the provided word, ignoring case.
    pub fn from_chars(word: &[char]) -> Self {
        Self {
            word: word.iter().copied().collect(),
            case_sensitive: false,
        }
    }

    /// Matches the provided word, ignoring case.
    pub fn from_char_string(word: CharString) -> Self {
        Self {
            word,
            case_sensitive: false,
        }
    }

    /// Matches the provided word, case-sensitive.
    pub fn new_exact(word: &'static str) -> Self {
        Self {
            word: word.chars().collect(),
            case_sensitive: true,
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

        let chars = token.span.get_content(source);
        if self.case_sensitive {
            chars == self.word.as_slice()
        } else {
            chars
                .iter()
                .zip(&self.word)
                .all(|(a, b)| a.eq_ignore_ascii_case(b))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Document, Span, patterns::DocPattern};

    use super::Word;

    #[test]
    fn fruit() {
        let doc = Document::new_markdown_default_curated("I ate a banana and an apple today.");

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
        let doc = Document::new_markdown_default_curated("I Ate A bAnaNa And aN apPlE today.");

        assert_eq!(
            Word::new("banana").find_all_matches_in_doc(&doc),
            vec![Span::new(6, 7)]
        );
        assert_eq!(
            Word::new_exact("banana").find_all_matches_in_doc(&doc),
            vec![]
        );
    }
}
