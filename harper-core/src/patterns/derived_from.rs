use crate::spell::{
    CanonicalWordId, CaseFoldedWordId, Dictionary, EitherWordId, MutableDictionary,
};

use super::Pattern;

/// A [Pattern] that looks for Word tokens that are either derived from a given word, or the word
/// itself.
///
/// For example, this will match "call" as well as "recall", "calling", etc.
pub struct DerivedFrom {
    word_id: EitherWordId,
}

impl DerivedFrom {
    pub fn new_from_str(word: &str) -> DerivedFrom {
        Self::new(EitherWordId::from_str_case_folded(word))
    }

    pub fn new_from_chars(word: &[char]) -> DerivedFrom {
        Self::new(EitherWordId::from_chars_case_folded(word))
    }

    pub fn new_from_str_exact(word: &str) -> DerivedFrom {
        Self::new(EitherWordId::from_str_canonical(word))
    }

    pub fn new_from_chars_exact(word: &[char]) -> DerivedFrom {
        Self::new(EitherWordId::from_chars_canonical(word))
    }

    pub fn new(word_id: EitherWordId) -> Self {
        Self { word_id }
    }
}

impl Pattern for DerivedFrom {
    fn matches(&self, tokens: &[crate::Token], source: &[char]) -> Option<usize> {
        let tok = tokens.first()?;
        let chars = tok.span.get_content(source);

        match self.word_id {
            EitherWordId::Canonical(canonical_word_id) => {
                let tok_derived_from_canonical = tok
                    .kind
                    .as_word()?
                    .as_ref()
                    .and_then(|meta| meta.derived_from)?
                    .canonical();

                if CanonicalWordId::from_word_chars(chars) == canonical_word_id
                    || tok_derived_from_canonical == canonical_word_id
                {
                    Some(1)
                } else {
                    None
                }
            }
            EitherWordId::CaseFolded(case_folded_word_id) => {
                let dict = MutableDictionary::curated();

                if CaseFoldedWordId::from_word_chars(chars) == case_folded_word_id
                    || dict
                        .get_word_metadata(chars)
                        .into_iter()
                        .filter_map(|word_meta| word_meta.derived_from)
                        .any(|word_id_pair| word_id_pair.case_folded() == case_folded_word_id)
                {
                    Some(1)
                } else {
                    None
                }
            }
        }
    }
}
