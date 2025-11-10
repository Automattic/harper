use crate::spell::WordId;

use super::Pattern;

/// A [Pattern] that looks for Word tokens that are either derived from a given word, or the word
/// itself.
///
/// For example, this will match "call" as well as "recall", "calling", etc.
pub struct DerivedFrom {
    word_id: WordId,
}

impl DerivedFrom {
    pub fn new_from_str(word: &str) -> DerivedFrom {
        Self::new(WordId::from_word_str(word))
    }

    pub fn new_from_chars(word: &[char]) -> DerivedFrom {
        Self::new(WordId::from_word_chars(word))
    }

    pub fn new(word_id: WordId) -> Self {
        Self { word_id }
    }
}

impl Pattern for DerivedFrom {
    fn matches(&self, tokens: &[crate::Token], source: &[char]) -> Option<usize> {
        let tok = tokens.first()?;
        let metadata = tok.kind.as_word()?.as_ref()?;

        if metadata.derived_from == Some(self.word_id) {
            return Some(1);
        }

        let chars = tok.span.get_content(source);
        let word_id = WordId::from_word_chars(chars);

        if word_id == self.word_id {
            return Some(1);
        }

        None
    }
}
