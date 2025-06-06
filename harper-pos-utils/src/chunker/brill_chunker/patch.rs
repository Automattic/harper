use hashbrown::HashSet;

use crate::{
    UPOS,
    patch_criteria::PatchCriteria,
    word_counter::{self, WordCounter},
};

#[derive(Debug, Clone)]
pub struct Patch {
    pub from: bool,
    pub criteria: PatchCriteria,
}

impl Patch {
    pub fn generate_candidate_patches(relevant_words: &WordCounter) -> Vec<Self> {
        let mut simple_candidates = Vec::new();

        simple_candidates.extend(Self::gen_simple_candidates().into_iter().map(|c| Patch {
            from: true,
            criteria: c,
        }));
        simple_candidates.extend(Self::gen_simple_candidates().into_iter().map(|c| Patch {
            from: false,
            criteria: c,
        }));

        let mut candidates = simple_candidates.clone();

        for base_c in simple_candidates {
            for word in relevant_words.iter_top_n_words(20) {
                for r in -3..3 {
                    candidates.push(Patch {
                        from: base_c.from,
                        criteria: PatchCriteria::Combined {
                            a: Box::new(PatchCriteria::WordIs {
                                relative: r,
                                word: word.to_string(),
                            }),
                            b: Box::new(base_c.criteria.clone()),
                        },
                    })
                }
            }
        }

        candidates
    }

    /// Candidates to be tested against a dataset during training.
    fn gen_simple_candidates() -> Vec<PatchCriteria> {
        use strum::IntoEnumIterator;

        let mut criteria = HashSet::new();

        for i in -4..=4 {
            criteria.insert(PatchCriteria::NounPhraseAt {
                is_np: true,
                relative: i,
            });
            criteria.insert(PatchCriteria::NounPhraseAt {
                is_np: false,
                relative: i,
            });
        }

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
