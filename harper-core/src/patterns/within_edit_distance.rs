use std::cell::RefCell;
use std::num::NonZeroUsize;

use super::Pattern;
use crate::{CharString, CharStringExt, Token};

use crate::edit_distance::edit_distance_min_alloc;

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

thread_local! {
    // To avoid allocating each call to `matches`.
    static BUFFERS: RefCell<(Vec<u8>, Vec<u8>)> = const { RefCell::new((Vec::new(), Vec::new())) };
}

impl Pattern for WithinEditDistance {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        let first = tokens.first()?;
        if !first.kind.is_word() {
            return None;
        }

        let content = first.span.get_content(source);

        BUFFERS.with_borrow_mut(|(buffer_a, buffer_b)| {
            let distance = edit_distance_min_alloc(
                &content.to_lower(),
                &self.word.to_lower(),
                buffer_a,
                buffer_b,
            );
            NonZeroUsize::new(if distance <= self.max_edit_dist { 1 } else { 0 })
        })
    }
}
