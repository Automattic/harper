// Similar to `harper-core/src/patterns/word_set.rs` but uses `Dictionary` and `WordId`

use std::sync::Arc;

use hashbrown::HashSet;
use smallvec::SmallVec;

use crate::{
    CharString, Span, Token,
    char_ext::CharExt,
    spell::{Dictionary, WordId},
};

/// A fast expression that matches against any of a set of provided words.
/// Uses WordId for dictionary words (fast lookup) and falls back to character comparison for OOV words.
pub struct FastWordSet {
    // For words in the dictionary we use WordId-based lookup (very fast)
    word_ids: HashSet<WordId>,
    // We only use the inefficient character comparison for words not in the dictionary
    oov_words: SmallVec<[CharString; 4]>,
}

impl FastWordSet {
    /// Create a new FastWordSet from a list of words using any dictionary reference.
    /// Words that exist in the dictionary are stored as WordIds for fast lookup.
    /// Words that don't exist in the dictionary are stored as CharStrings for fallback matching.
    pub fn new<D: Dictionary + ?Sized>(dict: &Arc<D>, words: &[&str]) -> Self {
        let mut word_ids = HashSet::new();
        let mut oov_words = SmallVec::new();

        for word in words {
            let word_id = WordId::from_word_str(word);

            // Check if the word exists in the dictionary
            if dict.get_word_from_id(&word_id).is_some() {
                word_ids.insert(word_id);
            } else {
                // Word not in dictionary, use character-based fallback
                let chars: CharString = word.chars().collect();
                if !oov_words.contains(&chars) {
                    oov_words.push(chars);
                }
            }
        }

        Self {
            word_ids,
            oov_words,
        }
    }

    /// Check if a token matches any word in this set.
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        if !token.kind.is_word() {
            return false;
        }

        let tok_chars = token.get_ch(source);
        let tok_id = WordId::from_word_chars(tok_chars);

        // First check the fast WordId lookup
        if self.word_ids.contains(&tok_id) {
            return true;
        }

        // Fall back to character-based comparison for OOV words
        for word in &self.oov_words {
            if tok_chars.len() != word.len() {
                continue;
            }

            let partial_match = tok_chars
                .iter()
                .map(CharExt::normalized)
                .zip(word.iter().map(CharExt::normalized))
                .all(|(a, b)| a.eq_ignore_ascii_case(&b));

            if partial_match {
                return true;
            }
        }

        false
    }
}

impl crate::expr::Expr for FastWordSet {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        if cursor >= tokens.len() {
            return None;
        }

        let token = &tokens[cursor];

        if self.matches_token(token, source) {
            Some(Span::new_with_len(cursor, 1))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::spell::FstDictionary;
    use crate::{Document, Span, expr::ExprExt};

    use super::FastWordSet;

    #[test]
    fn fruit() {
        let dict = FstDictionary::curated();
        let set = FastWordSet::new(&dict, &["banana", "apple", "orange"]);

        let doc = Document::new_markdown_default_curated("I ate a banana and an apple today.");

        let matches = set.iter_matches_in_doc(&doc).collect::<Vec<_>>();

        assert_eq!(matches, vec![Span::new(6, 7), Span::new(12, 13)]);
    }

    #[test]
    fn fruit_whack_capitalization() {
        let dict = FstDictionary::curated();
        let set = FastWordSet::new(&dict, &["banana", "apple", "orange"]);

        let doc = Document::new_markdown_default_curated("I Ate A bAnaNa And aN apPlE today.");

        let matches = set.iter_matches_in_doc(&doc).collect::<Vec<_>>();

        assert_eq!(matches, vec![Span::new(6, 7), Span::new(12, 13)]);
    }

    #[test]
    fn supports_typographic_apostrophes() {
        let dict = FstDictionary::curated();
        let set = FastWordSet::new(&dict, &["They're"]);

        let doc = Document::new_markdown_default_curated("They're");

        let matches = set.iter_matches_in_doc(&doc).collect::<Vec<_>>();

        assert_eq!(matches, vec![Span::new(0, 1)]);
    }

    #[test]
    fn oov_words_fallback() {
        let dict = FstDictionary::curated();
        // Include both dictionary words and made-up words
        let set = FastWordSet::new(&dict, &["hello", "blork", "world"]);

        let doc = Document::new_markdown_default_curated("hello blork world");

        let matches = set.iter_matches_in_doc(&doc).collect::<Vec<_>>();

        assert_eq!(
            matches,
            vec![Span::new(0, 1), Span::new(2, 3), Span::new(4, 5)]
        );
    }
}
