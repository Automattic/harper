use is_macro::Is;
use serde::{Deserialize, Serialize};

use crate::{ConjunctionData, NominalData, Number, PronounData, Punctuation, Quote, WordMetadata};

#[derive(
    Debug, Is, Clone, Copy, Serialize, Deserialize, Default, PartialOrd, Hash, Eq, PartialEq,
)]
#[serde(tag = "kind", content = "value")]
pub enum TokenKind {
    /// `None` if the word does not exist in the dictionary.
    Word(Option<WordMetadata>),
    Punctuation(Punctuation),
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
}

impl TokenKind {
    pub fn is_open_square(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::OpenSquare))
    }

    pub fn is_close_square(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::CloseSquare))
    }

    pub fn is_pipe(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Pipe))
    }

    /// Checks whether a token is word-like--meaning it is more complex than punctuation and can
    /// hold semantic meaning in the way a word does.
    pub fn is_word_like(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(..)
                | TokenKind::EmailAddress
                | TokenKind::Hostname
                | TokenKind::Number(..)
        )
    }

    pub fn is_possessive_nominal(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(Some(WordMetadata {
                nominal: Some(NominalData {
                    is_possessive: Some(true),
                    ..
                }),
                ..
            }))
        )
    }

    pub fn is_possessive_pronoun(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(Some(WordMetadata {
                pronoun: Some(PronounData {
                    is_possessive: Some(true),
                    ..
                }),
                ..
            }))
        )
    }

    pub fn is_proper_nominal(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(Some(WordMetadata {
                nominal: Some(NominalData {
                    is_proper: Some(true),
                    ..
                }),
                ..
            }))
        )
    }

    pub fn is_conjunction(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(Some(WordMetadata {
                conjunction: Some(ConjunctionData {}),
                ..
            }))
        )
    }

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

    pub fn is_currency(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Currency(..)))
    }

    pub fn is_article(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(Some(WordMetadata { article: true, .. }))
        )
    }

    pub fn is_ellipsis(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Ellipsis))
    }

    pub fn is_hyphen(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Hyphen))
    }

    pub fn is_adjective(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(Some(WordMetadata {
                adjective: Some(_),
                ..
            }))
        )
    }

    pub fn is_adverb(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(Some(WordMetadata {
                adverb: Some(_),
                ..
            }))
        )
    }

    pub fn is_swear(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(Some(WordMetadata {
                swear: Some(true),
                ..
            }))
        )
    }

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
            _ => *self,
        }
    }
}

impl TokenKind {
    /// Construct a [`TokenKind::Word`] with no metadata.
    pub fn blank_word() -> Self {
        Self::Word(None)
    }
}

impl TokenKind {
    pub fn as_mut_quote(&mut self) -> Option<&mut Quote> {
        self.as_mut_punctuation()?.as_mut_quote()
    }

    pub fn as_quote(&self) -> Option<&Quote> {
        self.as_punctuation()?.as_quote()
    }

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

    /// Used by `crate::parsers::CollapseIdentifiers`
    /// TODO: Separate this into two functions and add OR functionality to
    /// pattern matching
    pub fn is_case_separator(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Underscore))
            || matches!(self, TokenKind::Punctuation(Punctuation::Hyphen))
    }

    pub fn is_verb(&self) -> bool {
        let TokenKind::Word(Some(metadata)) = self else {
            return false;
        };

        metadata.is_verb()
    }

    pub fn is_linking_verb(&self) -> bool {
        let TokenKind::Word(Some(metadata)) = self else {
            return false;
        };

        metadata.is_linking_verb()
    }

    pub fn is_not_plural_nominal(&self) -> bool {
        let TokenKind::Word(Some(metadata)) = self else {
            return true;
        };

        metadata.is_not_plural_nominal()
    }

    pub fn is_not_plural_pronoun(&self) -> bool {
        let TokenKind::Word(Some(metadata)) = self else {
            return true;
        };

        metadata.is_not_plural_pronoun()
    }

    pub fn is_common_word(&self) -> bool {
        let TokenKind::Word(Some(metadata)) = self else {
            return true;
        };

        metadata.common
    }

    pub fn is_plural_nominal(&self) -> bool {
        let TokenKind::Word(Some(metadata)) = self else {
            return false;
        };

        metadata.is_plural_nominal()
    }

    pub fn is_plural_pronoun(&self) -> bool {
        let TokenKind::Word(Some(metadata)) = self else {
            return false;
        };

        metadata.is_plural_pronoun()
    }

    pub fn is_nominal(&self) -> bool {
        let TokenKind::Word(Some(metadata)) = self else {
            return false;
        };

        metadata.is_nominal()
    }

    pub fn is_pronoun(&self) -> bool {
        let TokenKind::Word(Some(metadata)) = self else {
            return false;
        };

        metadata.is_pronoun()
    }

    pub fn is_likely_homograph(&self) -> bool {
        let TokenKind::Word(Some(metadata)) = self else {
            return false;
        };

        metadata.is_likely_homograph()
    }

    pub fn is_comma(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Comma))
    }

    /// Checks whether the token is whitespace.
    pub fn is_whitespace(&self) -> bool {
        matches!(self, TokenKind::Space(_) | TokenKind::Newline(_))
    }
}
