use hashbrown::HashSet;

use crate::{UPOS, patch_criteria::PatchCriteria};

#[derive(Debug, Clone)]
pub struct Patch {
    pub from: bool,
    pub criteria: PatchCriteria,
}
impl Patch {
    pub fn generate_candidate_patches() -> Vec<Patch> {
        let mut candidates = Vec::new();

        candidates.extend(Self::gen_simple_candidates().into_iter().map(|c| Patch {
            from: true,
            criteria: c,
        }));
        candidates.extend(Self::gen_simple_candidates().into_iter().map(|c| Patch {
            from: false,
            criteria: c,
        }));

        candidates
    }

    /// Candidates to be tested against a dataset during training.
    fn gen_simple_candidates() -> Vec<PatchCriteria> {
        use strum::IntoEnumIterator;

        let mut criteria = HashSet::new();
        for upos in UPOS::iter() {
            criteria.insert(PatchCriteria::WordIsTaggedWith {
                relative: -1,
                is_tagged: upos,
            });
            criteria.insert(PatchCriteria::WordIsTaggedWith {
                relative: -2,
                is_tagged: upos,
            });
            criteria.insert(PatchCriteria::WordIsTaggedWith {
                relative: -3,
                is_tagged: upos,
            });
            criteria.insert(PatchCriteria::AnyWordIsTaggedWith {
                max_relative: -2,
                is_tagged: upos,
            });
            criteria.insert(PatchCriteria::AnyWordIsTaggedWith {
                max_relative: -3,
                is_tagged: upos,
            });
            criteria.insert(PatchCriteria::AnyWordIsTaggedWith {
                max_relative: -4,
                is_tagged: upos,
            });

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
