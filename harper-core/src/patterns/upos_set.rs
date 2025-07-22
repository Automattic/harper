use harper_brill::UPOS;
use smallvec::{SmallVec, ToSmallVec};

use crate::Token;

use super::Pattern;

pub struct UPOSSet {
    allowed_tags: SmallVec<[UPOS; 10]>,
}

impl UPOSSet {
    pub fn new(allowed: &[UPOS]) -> Self {
        Self {
            allowed_tags: allowed.to_smallvec(),
        }
    }

    /// Construct a new set with the tags needed to detect nominal words.
    pub fn new_nominal() -> Self {
        Self::new(&[UPOS::NOUN, UPOS::PROPN, UPOS::PRON])
    }
}

impl Pattern for UPOSSet {
    fn matches(&self, tokens: &[Token], _source: &[char]) -> Option<usize> {
        tokens.first()?.kind.as_word()?.as_ref().and_then(|w| {
            if self.allowed_tags.contains(&(w.pos_tag?)) {
                Some(1)
            } else {
                None
            }
        })
    }
}
