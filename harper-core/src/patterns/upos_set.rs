use harper_brill::UPOS;
use serde::{Deserialize, Serialize};
use smallvec::{SmallVec, ToSmallVec};

use crate::Token;

use super::Pattern;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UPOSSet {
    allowed_tags: SmallVec<[UPOS; 10]>,
    /// When set, match a *plausible* reading (argmax or a close runner-up in
    /// `pos_tag_topk`) rather than only the hair-thin argmax. Opt-in per use
    /// site so probability tolerance is confined to rules that genuinely need it
    /// (orthographic confusables on homographs); applying it globally over-fires
    /// guardrails that rely on the strict argmax.
    #[serde(default)]
    loose: bool,
}

impl UPOSSet {
    pub fn new(allowed: &[UPOS]) -> Self {
        Self {
            allowed_tags: allowed.to_smallvec(),
            loose: false,
        }
    }

    /// Probability-aware variant: matches if any allowed tag is a plausible
    /// reading of the token (the argmax or a close runner-up the tagger kept in
    /// `pos_tag_topk`). Use for confusable rules whose slot is a genuine
    /// homograph the tagger can only rank, not resolve.
    pub fn new_loose(allowed: &[UPOS]) -> Self {
        Self {
            allowed_tags: allowed.to_smallvec(),
            loose: true,
        }
    }
}

impl Pattern for UPOSSet {
    fn matches(&self, tokens: &[Token], _source: &[char]) -> Option<usize> {
        tokens.first()?.kind.as_word()?.as_ref().and_then(|w| {
            let hit = if self.loose {
                self.allowed_tags.iter().any(|t| w.could_be_pos(*t))
            } else {
                w.pos_tag.is_some_and(|p| self.allowed_tags.contains(&p))
            };
            if hit { Some(1) } else { None }
        })
    }
}
