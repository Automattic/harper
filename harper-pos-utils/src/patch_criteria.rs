use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::UPOS;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    RelativeWordCaps {
        relative: isize,
        is_capitalized: bool,
    },
    Combined {
        a: Box<PatchCriteria>,
        b: Box<PatchCriteria>,
    },
}

impl PatchCriteria {
    pub fn fulfils(&self, tokens: &[String], tags: &[Option<UPOS>], index: usize) -> bool {
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
            PatchCriteria::RelativeWordCaps {
                relative,
                is_capitalized,
            } => {
                let Some(index) = add(index, *relative) else {
                    return false;
                };

                tokens.get(index).is_some_and(|t| {
                    t.chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_uppercase() == *is_capitalized)
                })
            }
            Self::Combined { a, b } => {
                a.fulfils(tokens, tags, index) && b.fulfils(tokens, tags, index)
            }
        }
    }

    /// Candidates to be tested against a dataset during training.
    pub fn gen_candidates() -> Vec<PatchCriteria> {
        let mut criteria = Vec::new();
        for upos in UPOS::iter() {
            criteria.push(PatchCriteria::WordIsTaggedWith {
                relative: -1,
                is_tagged: upos,
            });
            criteria.push(PatchCriteria::WordIsTaggedWith {
                relative: -2,
                is_tagged: upos,
            });
            criteria.push(PatchCriteria::WordIsTaggedWith {
                relative: -3,
                is_tagged: upos,
            });
            criteria.push(PatchCriteria::AnyWordIsTaggedWith {
                max_relative: -2,
                is_tagged: upos,
            });
            criteria.push(PatchCriteria::AnyWordIsTaggedWith {
                max_relative: -3,
                is_tagged: upos,
            });
            criteria.push(PatchCriteria::AnyWordIsTaggedWith {
                max_relative: -4,
                is_tagged: upos,
            });

            criteria.push(PatchCriteria::RelativeWordCaps {
                relative: 0,
                is_capitalized: true,
            });
            criteria.push(PatchCriteria::RelativeWordCaps {
                relative: 0,
                is_capitalized: false,
            });
            criteria.push(PatchCriteria::RelativeWordCaps {
                relative: -1,
                is_capitalized: true,
            });
            criteria.push(PatchCriteria::RelativeWordCaps {
                relative: -1,
                is_capitalized: false,
            });
            criteria.push(PatchCriteria::RelativeWordCaps {
                relative: 1,
                is_capitalized: true,
            });
            criteria.push(PatchCriteria::RelativeWordCaps {
                relative: 1,
                is_capitalized: false,
            });

            for upos_b in UPOS::iter() {
                criteria.push(PatchCriteria::SandwichTaggedWith {
                    prev_word_tagged: upos,
                    post_word_tagged: upos_b,
                });

                criteria.push(PatchCriteria::Combined {
                    a: Box::new(PatchCriteria::WordIsTaggedWith {
                        relative: 1,
                        is_tagged: upos,
                    }),
                    b: Box::new(PatchCriteria::WordIsTaggedWith {
                        relative: -2,
                        is_tagged: upos_b,
                    }),
                })
            }
        }

        criteria
    }
}

fn add(u: usize, i: isize) -> Option<usize> {
    if i.is_negative() {
        u.checked_sub(i.wrapping_abs() as u32 as usize)
    } else {
        u.checked_add(i as usize)
    }
}
