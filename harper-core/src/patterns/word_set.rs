use std::num::NonZeroUsize;

use super::Pattern;
use smallvec::SmallVec;

use crate::{CharString, Token};

// A [`Pattern`] that matches against any of a set of provided words.
// For small sets of short words, it doesn't allocate.
//
// Note that any capitalization of the contained words will result in a match.
#[derive(Debug, Default, Clone)]
pub struct WordSet {
    words: SmallVec<[CharString; 4]>,
}

impl WordSet {
    pub fn add(&mut self, word: &str) {
        let chars = word.chars().collect();

        if !self.words.contains(&chars) {
            self.words.push(chars);
        }
    }

    pub fn contains(&self, word: &str) -> bool {
        self.words.contains(&word.chars().collect())
    }

    /// Create a new word set that matches against any word in the provided list.
    pub fn new(words: &[&'static str]) -> Self {
        let mut set = Self::default();

        for str in words {
            set.add(str);
        }

        set
    }
}

impl Pattern for WordSet {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        let tok = tokens.first()?;
        if !tok.kind.is_word() {
            return None;
        }

        let tok_chars = tok.span.get_content(source);

        for word in &self.words {
            if tok_chars.len() != word.len() {
                continue;
            }

            let partial_match = tok_chars
                .iter()
                .zip(word)
                .all(|(a, b)| a.eq_ignore_ascii_case(b));

            if partial_match {
                return NonZeroUsize::new(1);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{Document, Span, patterns::DocPattern};

    use super::WordSet;

    #[test]
    fn fruit() {
        let set = WordSet::new(&["banana", "apple", "orange"]);

        let doc = Document::new_markdown_default_curated("I ate a banana and an apple today.");

        let matches = set.find_all_matches_in_doc(&doc);

        assert_eq!(matches, vec![Span::new(6, 7), Span::new(12, 13)]);
    }

    #[test]
    fn fruit_whack_capitalization() {
        let set = WordSet::new(&["banana", "apple", "orange"]);

        let doc = Document::new_markdown_default_curated("I Ate A bAnaNa And aN apPlE today.");

        let matches = set.find_all_matches_in_doc(&doc);

        assert_eq!(matches, vec![Span::new(6, 7), Span::new(12, 13)]);
    }
}
