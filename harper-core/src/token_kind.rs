use harper_brill::UPOS;
use is_macro::Is;
use serde::{Deserialize, Serialize};

use crate::{Number, Punctuation, Quote, TokenKind::Word, WordMetadata};

#[derive(Debug, Is, Clone, Serialize, Deserialize, Default, PartialOrd, Hash, Eq, PartialEq)]
#[serde(tag = "kind", content = "value")]
pub enum TokenKind {
    /// `None` if the word does not exist in the dictionary.
    Word(Option<WordMetadata>),
    Punctuation(Punctuation),
    Decade,
    Number(Number),
    /// A sequence of " " spaces.
    Space(usize),
    /// A sequence of "\n" newlines
    Newline(usize),
    EmailAddress,
    Url,
    Hostname,
    /// A special token used for things like inline code blocks that should be
    /// ignored by all linters.
    #[default]
    Unlintable,
    ParagraphBreak,
    Regexish,
}

impl TokenKind {
    // Punctuation and symbol is-methods #1

    pub fn is_open_square(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::OpenSquare))
    }

    pub fn is_close_square(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::CloseSquare))
    }

    pub fn is_pipe(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Pipe))
    }

    // Miscellaneous is-methods #1

    /// Checks whether a token is word-like--meaning it is more complex than punctuation and can
    /// hold semantic meaning in the way a word does.
    pub fn is_word_like(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(..)
                | TokenKind::EmailAddress
                | TokenKind::Hostname
                | TokenKind::Decade
                | TokenKind::Number(..)
        )
    }

    // Word is-methods #1

    // Nominal is-methods (nouns and pronouns) #1

    pub fn is_possessive_nominal(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_possessive_nominal()
    }

    pub fn is_possessive_noun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_possessive_noun()
    }

    pub fn is_possessive_pronoun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_possessive_pronoun()
    }

    pub fn is_proper_noun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_proper_noun()
    }

    // Conjunction is-methods

    pub fn is_conjunction(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_conjunction()
    }

    // Miscellaneous is-methods #2

    pub(crate) fn is_chunk_terminator(&self) -> bool {
        if self.is_sentence_terminator() {
            return true;
        }

        match self {
            TokenKind::Punctuation(punct) => {
                matches!(
                    punct,
                    Punctuation::Comma | Punctuation::Quote { .. } | Punctuation::Colon
                )
            }
            _ => false,
        }
    }

    pub(crate) fn is_sentence_terminator(&self) -> bool {
        match self {
            TokenKind::Punctuation(punct) => [
                Punctuation::Period,
                Punctuation::Bang,
                Punctuation::Question,
            ]
            .contains(punct),
            TokenKind::ParagraphBreak => true,
            _ => false,
        }
    }

    // Punctuation and symbol is-methods #2

    pub fn is_currency(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Currency(..)))
    }

    // Word is-methods #2

    // Preposition is-methods

    pub fn is_preposition(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.preposition
    }

    // Punctuation and symbol is-methods #3

    pub fn is_ellipsis(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Ellipsis))
    }

    pub fn is_hyphen(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Hyphen))
    }

    // Word is-methods #3

    // Adjective is-methods

    pub fn is_adjective(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_adjective()
    }

    // Verb is-methods #1

    pub fn is_verb_lemma(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_verb_lemma()
    }

    pub fn is_verb_past_form(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_verb_past_form()
    }

    pub fn is_verb_progressive_form(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_verb_progressive_form()
    }

    pub fn is_verb_third_person_singular_present_form(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_verb_third_person_singular_present_form()
    }

    // Adverb is-methods

    pub fn is_adverb(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_adverb()
    }

    // Miscellaneous word is-methods #1

    pub fn is_swear(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_swear()
    }

    // Miscellaneous non-is methods #1

    /// Checks that `self` is the same enum variant as `other`, regardless of
    /// whether the inner metadata is also equal.
    pub fn matches_variant_of(&self, other: &Self) -> bool {
        self.with_default_data() == other.with_default_data()
    }

    /// Produces a copy of `self` with any inner data replaced with its default
    /// value. Useful for making comparisons on just the variant of the
    /// enum.
    pub fn with_default_data(&self) -> Self {
        match self {
            TokenKind::Word(_) => TokenKind::Word(Default::default()),
            TokenKind::Punctuation(_) => TokenKind::Punctuation(Default::default()),
            TokenKind::Number(..) => TokenKind::Number(Default::default()),
            TokenKind::Space(_) => TokenKind::Space(Default::default()),
            TokenKind::Newline(_) => TokenKind::Newline(Default::default()),
            _ => self.clone(),
        }
    }
}

impl TokenKind {
    // Miscellaneous non-is methods #2

    /// Construct a [`TokenKind::Word`] with no metadata.
    pub fn blank_word() -> Self {
        Self::Word(None)
    }
}

impl TokenKind {
    // Punctuation and symbol non-is methods

    pub fn as_mut_quote(&mut self) -> Option<&mut Quote> {
        self.as_mut_punctuation()?.as_mut_quote()
    }

    pub fn as_quote(&self) -> Option<&Quote> {
        self.as_punctuation()?.as_quote()
    }

    // Punctuation and symbol is-methods #4

    pub fn is_quote(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Quote(_)))
    }

    pub fn is_apostrophe(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Apostrophe))
    }

    pub fn is_period(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Period))
    }

    pub fn is_at(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::At))
    }

    // Miscellaneous is-methods #3

    /// Used by `crate::parsers::CollapseIdentifiers`
    /// TODO: Separate this into two functions and add OR functionality to
    /// pattern matching
    pub fn is_case_separator(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Underscore))
            || matches!(self, TokenKind::Punctuation(Punctuation::Hyphen))
    }

    // Word is-methods #4

    // Verb is-methods #2

    pub fn is_verb(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_verb()
    }

    pub fn is_auxiliary_verb(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_auxiliary_verb()
    }

    pub fn is_linking_verb(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_linking_verb()
    }

    // Nominal is-methods #2

    pub fn is_non_plural_nominal(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return true;
        };
        metadata.is_non_plural_nominal()
    }

    pub fn is_non_plural_noun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return true;
        };
        metadata.is_non_plural_noun()
    }

    pub fn is_non_plural_pronoun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return true;
        };
        metadata.is_non_plural_pronoun()
    }

    pub fn is_second_person_pronoun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return true;
        };
        metadata.is_second_person_pronoun()
    }

    pub fn is_third_person_pronoun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return true;
        };
        metadata.is_third_person_pronoun()
    }

    pub fn is_first_person_singular_pronoun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return true;
        };
        metadata.is_first_person_singular_pronoun()
    }

    pub fn is_first_person_plural_pronoun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return true;
        };
        metadata.is_first_person_plural_pronoun()
    }

    pub fn is_third_person_singular_pronoun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return true;
        };
        metadata.is_third_person_singular_pronoun()
    }

    pub fn is_third_person_plural_pronoun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return true;
        };
        metadata.is_third_person_plural_pronoun()
    }

    pub fn is_object_pronoun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return true;
        };
        metadata.is_object_pronoun()
    }

    // Miscellaneous word is-methods #2

    pub fn is_common_word(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return true;
        };
        metadata.common
    }

    // Nominal is-methods #3

    pub fn is_singular_nominal(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_singular_nominal()
    }

    pub fn is_singular_pronoun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_singular_pronoun()
    }

    pub fn is_singular_noun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_singular_noun()
    }

    pub fn is_plural_nominal(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_plural_nominal()
    }

    pub fn is_plural_pronoun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_plural_pronoun()
    }

    pub fn is_plural_noun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_plural_noun()
    }

    pub fn is_countable_noun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_countable_noun()
    }

    pub fn is_mass_noun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_mass_noun()
    }

    pub fn is_nominal(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_nominal()
    }

    pub fn is_noun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_noun()
    }

    pub fn is_pronoun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_pronoun()
    }

    pub fn is_reflexive_pronoun(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_reflexive_pronoun()
    }

    // Determiner is-methods

    pub fn is_determiner(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_determiner()
    }

    pub fn is_demonstrative_determiner(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_demonstrative_determiner()
    }

    pub fn is_possessive_determiner(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_possessive_determiner()
    }

    // Miscellaneous word is-methods #3

    pub fn is_likely_homograph(&self) -> bool {
        let Word(Some(metadata)) = self else {
            return false;
        };
        metadata.is_likely_homograph()
    }

    // Punctuation and symbol is-methods #5

    pub fn is_comma(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Comma))
    }

    // Miscellaneous is-methods #4

    /// Checks whether the token is whitespace.
    pub fn is_whitespace(&self) -> bool {
        matches!(self, TokenKind::Space(_) | TokenKind::Newline(_))
    }

    pub fn is_upos(&self, upos: UPOS) -> bool {
        let Some(Some(meta)) = self.as_word() else {
            return false;
        };

        meta.pos_tag == Some(upos)
    }
}

#[cfg(test)]
mod tests {
    use crate::Document;

    #[test]
    fn car_is_singular_noun() {
        let doc = Document::new_plain_english_curated("car");
        let tk = &doc.tokens().next().unwrap().kind;
        assert!(tk.is_singular_noun());
    }

    #[test]
    fn traffic_is_mass_noun() {
        let doc = Document::new_plain_english_curated("traffic");
        let tk = &doc.tokens().next().unwrap().kind;
        assert!(tk.is_mass_noun());
    }
}
