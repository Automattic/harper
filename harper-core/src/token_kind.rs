use is_macro::Is;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::{ConjunctionData, NounData, Punctuation, Quote, WordMetadata};

#[derive(
    Debug, Is, Clone, Copy, Serialize, Deserialize, Default, PartialOrd, Hash, Eq, PartialEq,
)]
#[serde(tag = "kind", content = "value")]
pub enum TokenKind {
    Word(WordMetadata),
    Punctuation(Punctuation),
    Number(OrderedFloat<f64>, Option<NumberSuffix>),
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

    pub fn is_pronoun(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(WordMetadata {
                noun: Some(NounData {
                    is_pronoun: Some(true),
                    ..
                }),
                ..
            })
        )
    }

    pub fn is_conjunction(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(WordMetadata {
                conjunction: Some(ConjunctionData {}),
                ..
            })
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
        matches!(self, TokenKind::Word(WordMetadata { article: true, .. }))
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
            TokenKind::Word(WordMetadata {
                adjective: Some(_),
                ..
            })
        )
    }

    pub fn is_adverb(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(WordMetadata {
                adverb: Some(_),
                ..
            })
        )
    }

    pub fn is_swear(&self) -> bool {
        matches!(
            self,
            TokenKind::Word(WordMetadata {
                swear: Some(true),
                ..
            })
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
            TokenKind::Number(..) => TokenKind::Number(Default::default(), Default::default()),
            TokenKind::Space(_) => TokenKind::Space(Default::default()),
            TokenKind::Newline(_) => TokenKind::Newline(Default::default()),
            _ => *self,
        }
    }
}

impl TokenKind {
    /// Construct a [`TokenKind::Word`] with no (default) metadata.
    pub fn blank_word() -> Self {
        Self::Word(WordMetadata::default())
    }
}

#[derive(
    Debug, Serialize, Deserialize, Default, PartialEq, PartialOrd, Clone, Copy, Is, Hash, Eq,
)]
pub enum NumberSuffix {
    #[default]
    Th,
    St,
    Nd,
    Rd,
}

impl NumberSuffix {
    pub fn correct_suffix_for(number: impl Into<f64>) -> Option<Self> {
        let number = number.into();

        if number < 0.0 || number - number.floor() > f64::EPSILON || number > u64::MAX as f64 {
            return None;
        }

        let integer = number as u64;

        if let 11..=13 = integer % 100 {
            return Some(Self::Th);
        };

        match integer % 10 {
            0 => Some(Self::Th),
            1 => Some(Self::St),
            2 => Some(Self::Nd),
            3 => Some(Self::Rd),
            4 => Some(Self::Th),
            5 => Some(Self::Th),
            6 => Some(Self::Th),
            7 => Some(Self::Th),
            8 => Some(Self::Th),
            9 => Some(Self::Th),
            _ => None,
        }
    }

    pub fn to_chars(self) -> Vec<char> {
        match self {
            NumberSuffix::Th => vec!['t', 'h'],
            NumberSuffix::St => vec!['s', 't'],
            NumberSuffix::Nd => vec!['n', 'd'],
            NumberSuffix::Rd => vec!['r', 'd'],
        }
    }

    /// Check the first several characters in a buffer to see if it matches a
    /// number suffix.
    pub fn from_chars(chars: &[char]) -> Option<Self> {
        if chars.len() < 2 {
            return None;
        }

        match (chars[0], chars[1]) {
            ('t', 'h') => Some(NumberSuffix::Th),
            ('T', 'h') => Some(NumberSuffix::Th),
            ('t', 'H') => Some(NumberSuffix::Th),
            ('T', 'H') => Some(NumberSuffix::Th),
            ('s', 't') => Some(NumberSuffix::St),
            ('S', 't') => Some(NumberSuffix::St),
            ('s', 'T') => Some(NumberSuffix::St),
            ('S', 'T') => Some(NumberSuffix::St),
            ('n', 'd') => Some(NumberSuffix::Nd),
            ('N', 'd') => Some(NumberSuffix::Nd),
            ('n', 'D') => Some(NumberSuffix::Nd),
            ('N', 'D') => Some(NumberSuffix::Nd),
            ('r', 'd') => Some(NumberSuffix::Rd),
            ('R', 'd') => Some(NumberSuffix::Rd),
            ('r', 'D') => Some(NumberSuffix::Rd),
            ('R', 'D') => Some(NumberSuffix::Rd),
            _ => None,
        }
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
        let TokenKind::Word(metadata) = self else {
            return false;
        };

        metadata.is_verb()
    }

    pub fn is_linking_verb(&self) -> bool {
        let TokenKind::Word(metadata) = self else {
            return false;
        };

        metadata.is_linking_verb()
    }

    pub fn is_not_pronoun_noun(&self) -> bool {
        let TokenKind::Word(metadata) = self else {
            return true;
        };

        metadata.is_not_pronoun_noun()
    }

    pub fn is_not_plural_noun(&self) -> bool {
        let TokenKind::Word(metadata) = self else {
            return true;
        };

        metadata.is_not_plural_noun()
    }

    pub fn is_common_word(&self) -> bool {
        let TokenKind::Word(metadata) = self else {
            return true;
        };

        metadata.common
    }

    pub fn is_plural_noun(&self) -> bool {
        let TokenKind::Word(metadata) = self else {
            return false;
        };

        metadata.is_plural_noun()
    }

    pub fn is_noun(&self) -> bool {
        let TokenKind::Word(metadata) = self else {
            return false;
        };

        metadata.is_noun()
    }

    pub fn is_likely_homograph(&self) -> bool {
        let TokenKind::Word(metadata) = self else {
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
