use serde::{Deserialize, Serialize};

use crate::UPOS;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum PatchCriteria {
    WordIsTaggedWith {
        /// Which token to inspect.
        relative: isize,
        is_tagged: UPOS,
    },
    AnyWordIsTaggedWith {
        /// The farthest relative index to look
        max_relative: isize,
        is_tagged: UPOS,
    },
    SandwichTaggedWith {
        prev_word_tagged: UPOS,
        post_word_tagged: UPOS,
    },
    WordIs {
        relative: isize,
        word: String,
    },
    /// Not applicable to the Brill Tagger, only the chunker
    NounPhraseAt {
        is_np: bool,
        relative: isize,
    },
    Combined {
        a: Box<PatchCriteria>,
        b: Box<PatchCriteria>,
    },
}

impl PatchCriteria {
    pub fn fulfils(
        &self,
        tokens: &[String],
        tags: &[Option<UPOS>],
        np_flags: &[bool],
        index: usize,
    ) -> bool {
        match self {
            PatchCriteria::WordIsTaggedWith {
                relative,
                is_tagged,
            } => {
                let Some(index) = add(index, *relative) else {
                    return false;
                };

                tags.get(index)
                    .copied()
                    .flatten()
                    .is_some_and(|t| t == *is_tagged)
            }
            PatchCriteria::AnyWordIsTaggedWith {
                max_relative: relative,
                is_tagged,
            } => {
                let Some(farthest_index) = add(index, *relative) else {
                    return false;
                };

                (farthest_index.min(index)..farthest_index.max(index)).any(|i| {
                    tags.get(i)
                        .copied()
                        .flatten()
                        .is_some_and(|t| t == *is_tagged)
                })
            }
            PatchCriteria::SandwichTaggedWith {
                prev_word_tagged,
                post_word_tagged,
            } => {
                if index == 0 {
                    return false;
                }

                let prev_i = index - 1;
                let post_i = index + 1;

                tags.get(prev_i)
                    .copied()
                    .flatten()
                    .is_some_and(|t| t == *prev_word_tagged)
                    && tags
                        .get(post_i)
                        .copied()
                        .flatten()
                        .is_some_and(|t| t == *post_word_tagged)
            }
            Self::WordIs { relative, word } => {
                let Some(index) = add(index, *relative) else {
                    return false;
                };

                tokens.get(index).is_some_and(|w| {
                    w.chars()
                        .zip(word.chars())
                        .all(|(a, b)| a.eq_ignore_ascii_case(&b))
                })
            }

            Self::NounPhraseAt { is_np, relative } => {
                let Some(index) = add(index, *relative) else {
                    return false;
                };

                np_flags.get(index).is_some_and(|f| *is_np == *f)
            }
            Self::Combined { a, b } => {
                a.fulfils(tokens, tags, np_flags, index) && b.fulfils(tokens, tags, np_flags, index)
            }
        }
    }
}

fn add(u: usize, i: isize) -> Option<usize> {
    if i.is_negative() {
        u.checked_sub(i.wrapping_abs() as u32 as usize)
    } else {
        u.checked_add(i as usize)
    }
}
