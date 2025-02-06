use super::Pattern;
use crate::{CharString, CharStringExt, Token, TokenKind};

use crate::edit_distance::edit_distance;

/// A [`Pattern`] that matches single words within a certain edit distance of a given word.
pub struct WithinEditDistance {
    word: CharString,
    max_edit_dist: u8,
}

impl WithinEditDistance {
    pub fn new(word: CharString, max_edit_dist: u8) -> Self {
        Self {
            word,
            max_edit_dist,
        }
    }

    pub fn from_str(word: &str, edit_dist: u8) -> Self {
        let chars = word.chars().collect();

        Self::new(chars, edit_dist)
    }
}

impl Pattern for WithinEditDistance {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize {
        let Some(first) = tokens.first() else {
            return 0;
        };

        let TokenKind::Word(_) = first.kind else {
            return 0;
        };

        let content = first.span.get_content(source);

        if edit_distance(&content.to_lower(), &self.word.to_lower()) <= self.max_edit_dist {
            1
        } else {
            0
        }
    }
}
