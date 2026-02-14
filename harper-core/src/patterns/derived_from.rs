use crate::spell::{CanonicalWordId, CaseFoldedWordId};

use super::Pattern;

/// A [Pattern] that looks for Word tokens that are either derived from a given word, or the word
/// itself.
///
/// For example, this will match "call" as well as "recall", "calling", etc.
pub struct DerivedFrom {
    word_id: CanonicalWordId,
}

impl DerivedFrom {
    pub fn new_from_str(word: &str) -> DerivedFrom {
        Self::new(CanonicalWordId::from_word_str(word))
    }

    pub fn new_from_chars(word: &[char]) -> DerivedFrom {
        Self::new(CanonicalWordId::from_word_chars(word))
    }

    pub fn new(word_id: CanonicalWordId) -> Self {
        Self { word_id }
    }
}

impl Pattern for DerivedFrom {
    fn matches(&self, tokens: &[crate::Token], source: &[char]) -> Option<usize> {
        let tok = tokens.first()?;
        let chars = tok.span.get_content(source);

        let is_exact_match = CanonicalWordId::from_word_chars(chars) == self.word_id
            || CaseFoldedWordId::from_word_chars(chars) == self.word_id.as_case_folded();

        if is_exact_match
            || tok
                .kind
                .as_word()?
                .as_ref()
                .is_some_and(|meta| meta.derived_from.contains(self.word_id))
        {
            Some(1)
        } else {
            None
        }
    }
}
