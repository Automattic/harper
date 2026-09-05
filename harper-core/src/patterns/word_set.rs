use super::SingleTokenPattern;
use smallvec::SmallVec;

use crate::{CharString, CharStringExt, Token, char_ext::CharExt};

/// A [`super::Pattern`] that matches against any of a set of provided words.
/// For small sets of short words, it doesn't allocate.
///
/// Note that any capitalization of the contained words will result in a match.
#[derive(Debug, Default, Clone)]
pub struct WordSet {
    words: SmallVec<[CharString; 4]>,
}

impl WordSet {
    pub fn add(&mut self, word: &str) {
        if !self.contains(word) {
            self.words.push(
                word.chars()
                    .map(|c| c.normalized().to_ascii_lowercase())
                    .collect(),
            );
        }
    }

    pub fn add_chars(&mut self, chars: &[char]) {
        if !self.contains_chars(chars) {
            self.words.push(
                chars
                    .iter()
                    .map(|c| c.normalized().to_ascii_lowercase())
                    .collect(),
            );
        }
    }

    pub fn contains(&self, word: &str) -> bool {
        self.words.iter().any(|w| w.as_ref().eq_str_lenient(word))
    }

    pub fn contains_chars(&self, chars: &[char]) -> bool {
        self.words.iter().any(|w| w.as_ref().eq_ch_lenient(chars))
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

impl SingleTokenPattern for WordSet {
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        if !token.kind.is_word() {
            return false;
        }

        let tok_chars = token.get_ch(source);

        for word in &self.words {
            if tok_chars.len() != word.len() {
                continue;
            }

            let partial_match = tok_chars
                .iter()
                .zip(word.iter())
                .all(|(a, b)| a.normalized().eq_ignore_ascii_case(&b.normalized()));

            if partial_match {
                return true;
            }
        }

        false
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

    #[test]
    fn supports_typographic_apostrophes() {
        let set = WordSet::new(&["They're"]);

        let doc = Document::new_markdown_default_curated("They’re");

        let matches = set.find_all_matches_in_doc(&doc);

        assert_eq!(matches, vec![Span::new(0, 1)]);
    }
}
