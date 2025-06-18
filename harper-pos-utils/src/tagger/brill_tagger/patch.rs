#[cfg(feature = "training")]
use crate::tagger::error_counter::ErrorCounter;
use crate::{UPOS, patch_criteria::PatchCriteria};
#[cfg(feature = "training")]
use hashbrown::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub from: UPOS,
    pub to: UPOS,
    pub criteria: PatchCriteria,
}

#[cfg(feature = "training")]
impl Patch {
    /// Given a list of tagging errors, generate a collection of candidate patches that _might_ fix
    /// them. Training involves determining which candidates actually work.
    pub fn generate_candidate_patches(error_counter: &ErrorCounter) -> Vec<Patch> {
        let mut candidates = Vec::new();

        for key in error_counter.error_counts.keys() {
            candidates.extend(Self::gen_simple_candidates().into_iter().map(|c| Patch {
                from: key.was_tagged,
                to: key.correct_tag,
                criteria: c,
            }));

            for c in &Self::gen_simple_candidates() {
                for word in error_counter.word_counts.iter_top_n_words(10) {
                    for r in -3..3 {
                        candidates.push(Patch {
                            from: key.was_tagged,
                            to: key.correct_tag,
                            criteria: PatchCriteria::Combined {
                                a: Box::new(PatchCriteria::WordIs {
                                    relative: r,
                                    word: word.to_string(),
                                }),
                                b: Box::new(c.clone()),
                            },
                        })
                    }
                }
            }
        }

        candidates
    }

    /// Candidates to be tested against a dataset during training.
    fn gen_simple_candidates() -> Vec<PatchCriteria> {
        use strum::IntoEnumIterator;

        let mut criteria = HashSet::new();
        for upos in UPOS::iter() {
            for i in -4..=4 {
                criteria.insert(PatchCriteria::WordIsTaggedWith {
                    relative: i,
                    is_tagged: upos,
                });
            }

            for i in -4..=4 {
                criteria.insert(PatchCriteria::AnyWordIsTaggedWith {
                    max_relative: i,
                    is_tagged: upos,
                });
            }

            for upos_b in UPOS::iter() {
                criteria.insert(PatchCriteria::SandwichTaggedWith {
                    prev_word_tagged: upos,
                    post_word_tagged: upos_b,
                });

                criteria.insert(PatchCriteria::Combined {
                    a: Box::new(PatchCriteria::WordIsTaggedWith {
                        relative: 1,
                        is_tagged: upos,
                    }),
                    b: Box::new(PatchCriteria::WordIsTaggedWith {
                        relative: -2,
                        is_tagged: upos_b,
                    }),
                });
            }
        }

        criteria.into_iter().collect()
    }
}
